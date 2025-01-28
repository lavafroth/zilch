use retry::{delay::Fixed, retry};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};
use tauri::{AppHandle, Emitter, Listener};
mod apk;

use adb_client::ADBDeviceExt;
use adb_client::ADBUSBDevice;

#[tauri::command]
async fn scan(app: tauri::AppHandle) -> Result<(), String> {
    if DEV.0.get().is_none() {
        DEV.scan().map_err(|e| e.to_string())?;
    }
    app.emit("device-ready", true)
        .map_err(|_| "failed to emit a message stating the device is ready".to_string())?;
    Ok(())
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Package {
    id: String,
    #[serde(skip)]
    path: String,
    name: Option<String>,
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Package {}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl Package {
    fn many_from(s: &str) -> Vec<Self> {
        s.lines()
            .map(|line| line.strip_prefix("package:").unwrap_or(line).to_string())
            .map(|line| match line.rsplit_once("=") {
                Some((path, id)) => Package {
                    name: None,
                    id: id.to_string(),
                    path: path.to_string(),
                },
                None => Package {
                    name: None,
                    id: line,
                    path: String::default(),
                },
            })
            .collect()
    }
}

#[tauri::command]
async fn list_packages(app: AppHandle) -> Result<(), String> {
    // Release the device mutex after each operation so that
    // competing events are not blocked
    let pkgs = {
        let Ok(mut dev) = DEV
            .0
            .get()
            .expect("could not unwrap handle after initialization; something terrible has happened")
            .try_lock()
        else {
            return Ok(());
        };
        let mut buffer = Vec::with_capacity(4096);
        dev.device
            .shell_command(&["pm list packages -f"], &mut buffer)
            .map_err(|e| e.to_string())?;
        let pkgs: Vec<_> = Package::many_from(&std::str::from_utf8(&buffer).unwrap());

        for pkg in pkgs.iter() {
            if !dev.pkgs.contains_key(&pkg.id) {
                dev.pkgs.insert(pkg.id.clone(), pkg.clone());
            }
        }
        app.emit(
            "packages-updated",
            dev.pkgs.values().cloned().collect::<Vec<_>>(),
        )
        .map_err(|_| "failed to send indexing message to the frontend".to_string())?;
        pkgs
    };

    for pkg in pkgs.iter() {
        let Ok(mut dev) = DEV
            .0
            .get()
            .expect("could not unwrap handle after initialization; something terrible has happened")
            .try_lock()
        else {
            return Ok(());
        };
        if dev
            .pkgs
            .get(&pkg.id)
            .expect("package does not exist in package set despite being added previously")
            .name
            .is_some()
        {
            continue;
        }
        let mut pulled = Vec::with_capacity(4096);
        let label = match dev.device.pull(&pkg.path, &mut pulled) {
            Ok(_) => {
                let label = match apk::label(&pulled) {
                    Ok(label) => label,
                    Err(e) => {
                        eprintln!("failed to get app label for package: {}: {e}", pkg.id);
                        None
                    }
                };
                label.unwrap_or("No name".to_string())
            }
            Err(e) => {
                eprintln!("failed to pull apk from device for {}: {e}", pkg.id);
                "No name".to_string()
            }
        };
        dev.pkgs
            .get_mut(&pkg.id)
            .expect("package does not exist in package set despite being added previously")
            .name
            .replace(label);
        app.emit(
            "packages-updated",
            pkgs.iter()
                .map(|pkg| dev.pkgs.get(&pkg.id).unwrap())
                .collect::<Vec<_>>(),
        )
        .map_err(|_| "failed to send updated package list to the frontend".to_string())?;
    }

    let Ok(dev) = DEV
        .0
        .get()
        .expect("could not unwrap handle after initialization; something terrible has happened")
        .try_lock()
    else {
        return Ok(());
    };
    app.emit(
        "packages-updated",
        pkgs.iter()
            .map(|pkg| dev.pkgs.get(&pkg.id).unwrap())
            .collect::<Vec<_>>(),
    )
    .map_err(|_| "failed to send updated package list to the frontend".to_string())?;
    Ok(())
}

async fn uninstall_packages(pkgs: Vec<String>) -> Result<(), String> {
    let mut dev = loop {
        if let Ok(dev) = DEV
            .0
            .get()
            .expect("could not unwrap handle after initialization; something terrible has happened")
            .try_lock()
        {
            break dev;
        }
    };

    for pkg in pkgs {
        let uninstall_command = format!("pm uninstall --user 0 -k {pkg}");
        // println!("I am about to run {uninstall_command:?}");
        let mut buffer = Vec::with_capacity(256);
        dev.device
            .shell_command(&[&uninstall_command], &mut buffer)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub struct Device {
    device: ADBUSBDevice,
    pkgs: BTreeMap<String, Package>,
}

pub struct DeviceLock(OnceLock<Mutex<Device>>);

impl DeviceLock {
    pub fn scan(&self) -> Result<(), String> {
        loop {
            let Ok(device) = retry(Fixed::from_millis(1000).take(5), || {
                ADBUSBDevice::autodetect()
            }) else {
                eprintln!("could not find any devices");
                continue;
            };
            self.0
                .set(Mutex::new(Device {
                    device,
                    pkgs: BTreeMap::default(),
                }))
                .map_err(|_| "unable to set global device handle".to_string())?;
            return Ok(());
        }
    }
}

static DEV: DeviceLock = DeviceLock(OnceLock::new());
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.listen("uninstall", |event| {
                if let Ok(payload) = serde_json::from_str::<Vec<String>>(event.payload()) {
                    tauri::async_runtime::spawn(uninstall_packages(payload));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![scan, list_packages])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
