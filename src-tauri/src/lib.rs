use retry::{delay::Fixed, retry};
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use tauri::Emitter;
mod apk;

use adb_client::ADBUSBDevice;

#[tauri::command]
async fn scan(app: tauri::AppHandle) -> Result<(), String> {
    if DEV.0.get().is_none() {
        DEV.scan().map_err(|e| format!("{e}"))?;
    }
    app.emit("device-ready", true)
        .map_err(|_| "failed to emit a message stating the device is ready".to_string())?;
    Ok(())
}

#[derive(Serialize, Clone)]
pub struct Package {
    id: String,
    #[serde(skip)]
    path: String,
    name: Option<String>,
}

#[tauri::command]
async fn list_packages(app: tauri::AppHandle) -> Result<(), String> {
    let mut dev = DEV
        .0
        .get()
        .expect("could not unwrap handle after initialization; something terrible has happened")
        .lock()
        .expect("could not unwrap handle after initialization; something terrible has happened");
    let mut pkgs: Vec<_> = dev
        .shell("pm list packages -f")
        .map_err(|e| e.to_string())?
        .lines()
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
        .collect();

    app.emit("packages-updated", pkgs.clone())
        .map_err(|_| "failed to send indexing message to the frontend".to_string())?;

    for i in 0..pkgs.len() {
        {
            let pkg = pkgs.get_mut(i).unwrap();
            let pulled = match dev.pull(&pkg.path) {
                Ok(pulled) => pulled,
                Err(e) => {
                    eprintln!("failed to pull apk from device for {}: {e}", pkg.id);
                    pkg.name.replace("No name".to_string());
                    app.emit("packages-updated", pkgs.clone()).map_err(|_| {
                        "failed to send updated package list to the frontend".to_string()
                    })?;
                    continue;
                }
            };
            let label = match apk::label(&pulled) {
                Ok(label) => label,
                Err(e) => {
                    eprintln!("failed to get app label for package: {}: {e}", pkg.id);
                    None
                }
            };
            println!("{} = {:?}", pkg.id, label);
            pkg.name.replace(label.unwrap_or("No name".to_string()));
        }
        app.emit("packages-updated", pkgs.clone())
            .map_err(|_| "failed to send updated package list to the frontend".to_string())?;
    }

    app.emit("packages-updated", pkgs)
        .map_err(|_| "failed to send updated package list to the frontend".to_string())?;
    Ok(())
}

pub struct DeviceLock(OnceLock<Mutex<ADBUSBDevice>>);

impl DeviceLock {
    pub fn scan(&self) -> Result<(), String> {
        loop {
            let Some((vid, pid)) = adb_client::search_adb_devices() else {
                continue;
            };

            println!("I found one");
            let Ok(mut device) = retry(Fixed::from_millis(1000).take(5), || {
                println!("Trying to connect to ({vid}, {pid})");
                ADBUSBDevice::new(vid, pid, None)
            }) else {
                eprintln!("the device took too long to respond, ignoring");
                continue;
            };
            let Ok(_) = retry(Fixed::from_millis(1000).take(5), || {
                println!("Trying to send connect message to ({vid}, {pid})");
                device.send_connect()
            }) else {
                eprintln!("the device took too long to respond, ignoring");
                continue;
            };
            self.0
                .set(Mutex::new(device))
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
        .invoke_handler(tauri::generate_handler![scan, list_packages])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
