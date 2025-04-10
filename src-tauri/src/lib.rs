use std::{
    collections::BTreeMap,
    sync::{RwLock, RwLockWriteGuard},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Listener};
mod apk;
mod package;
use adb_client::ADBDeviceExt;
use adb_client::ADBUSBDevice;
use package::Package;

#[tauri::command]
async fn scan(app: tauri::AppHandle) -> Result<(), String> {
    DEV.scan()?;
    app.emit("device-ready", true)
        .map_err(|_| "failed to emit a message stating the device is ready".to_string())?;
    Ok(())
}

#[tauri::command]
async fn list_packages(app: AppHandle) -> Result<(), String> {
    // Release the device mutex after each operation so that
    // competing events are not blocked
    let pkgs = {
        let mut try_get = DEV.try_get()?;
        let dev = try_get.as_mut().unwrap();
        let mut buffer = Vec::with_capacity(4096);
        dev.device
            .shell_command(&["pm list packages -f"], &mut buffer)
            .map_err(|e| e.to_string())?;
        let pkgs: Vec<_> = Package::many_from(std::str::from_utf8(&buffer).unwrap());

        for pkg in pkgs.iter() {
            if !dev.pkgs.contains_key(&pkg.id) {
                dev.pkgs.insert(pkg.id.clone(), pkg.clone());
            }
        }
        app.emit("packages-updated", pkgs.clone())
            .map_err(|_| "failed to send indexing message to the frontend".to_string())?;
        pkgs
    };

    for pkg in pkgs.iter() {
        let mut try_get = DEV.try_get()?;
        let dev = try_get.as_mut().unwrap();
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

    Ok(())
}

async fn uninstall_packages(pkgs: Vec<String>) -> Result<(), String> {
    let mut try_get = DEV.try_get()?;
    let mut maybe_device = try_get.as_mut();
    while maybe_device.is_none() {
        try_get = DEV.try_get()?;
        maybe_device = try_get.as_mut();
    }

    let dev = maybe_device.unwrap();

    for pkg in pkgs {
        let path = &dev.pkgs.get(&pkg).unwrap().path;
        if path.is_empty() {
            eprintln!("oh no! the app has no path for the apk, anyways: proceeding to uninstall");
        } else {
            let copy_command = format!("cp {path} /data/local/tmp/{pkg}.apk");
            let mut buffer = Vec::with_capacity(256);
            dev.device
                .shell_command(&[&copy_command], &mut buffer)
                .map_err(|e| e.to_string())?;
        }
        let uninstall_command = format!("pm uninstall --user 0 -k {pkg}");
        let mut buffer = Vec::with_capacity(256);
        dev.device
            .shell_command(&[&uninstall_command], &mut buffer)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

async fn disable_packages(pkgs: Vec<String>) -> Result<(), String> {
    let mut try_get = DEV.try_get()?;
    let mut maybe_device = try_get.as_mut();
    while maybe_device.is_none() {
        try_get = DEV.try_get()?;
        maybe_device = try_get.as_mut();
    }

    let dev = maybe_device.unwrap();

    for pkg in pkgs {
        let disable_command = format!("pm disable {pkg}");
        // println!("I am about to run {disable_command:?}");
        let mut buffer = Vec::with_capacity(256);
        dev.device
            .shell_command(&[&disable_command], &mut buffer)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

async fn revert_packages(pkgs: Vec<String>) -> Result<(), String> {
    let mut try_get = DEV.try_get()?;
    let mut maybe_device = try_get.as_mut();
    while maybe_device.is_none() {
        try_get = DEV.try_get()?;
        maybe_device = try_get.as_mut();
    }

    let dev = maybe_device.unwrap();

    for pkg in pkgs {
        let revert_command = format!("package install-existing {pkg}");
        eprintln!("I am about to run {revert_command:?}");
        let mut buffer = Vec::with_capacity(256);

        dev.device
            .shell_command(&[&revert_command], &mut buffer)
            .map_err(|e| e.to_string())?;

        let output = std::str::from_utf8(&buffer).unwrap();
        if !output.contains("inaccessible or not found") {
            return Ok(());
        }

        let revert_command = format!("pm install -r --user 0 /data/local/tmp/{pkg}.apk");
        eprintln!("I am about to run {revert_command:?}");
        buffer.clear();
        dev.device
            .shell_command(&[&revert_command], &mut buffer)
            .map_err(|e| e.to_string())?;

        let output = String::from_utf8(buffer).unwrap();

        eprintln!("output: {output:?}");
        if output.contains("Unable to open file") {
            return Err(
                "failed to revert: please soil your pants, this is uncharted territory".to_string(),
            );
        }
    }
    Ok(())
}

pub struct ZilchDevice {
    device: ADBUSBDevice,
    pkgs: BTreeMap<String, Package>,
}

pub struct DeviceLock {
    inner: RwLock<Option<ZilchDevice>>,
}

impl DeviceLock {
    pub fn scan(&self) -> Result<(), String> {
        loop {
            let Ok(device) = ADBUSBDevice::autodetect() else {
                println!("looking ...");
                thread::sleep(Duration::from_secs(3));
                continue;
            };

            let physical_device = ZilchDevice {
                device,
                pkgs: BTreeMap::default(),
            };

            let mut guard = self
                .inner
                .write()
                .map_err(|_| "failed to get write handle on zilch devicelock".to_string())?;
            guard.replace(physical_device);
            return Ok(());
        }
    }

    pub fn try_get(
        &self,
    ) -> Result<RwLockWriteGuard<'_, std::option::Option<ZilchDevice>>, std::string::String> {
        let zilch_device = self
            .inner
            .try_write()
            .map_err(|_| "failed to get write handle on zilch devicelock".to_string())?;
        if zilch_device.is_none() {
            return Err("device slot is empty".to_string());
        }
        Ok(zilch_device)
    }
}

static DEV: DeviceLock = DeviceLock {
    inner: RwLock::new(None),
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.listen("uninstall", |event| {
                if let Ok(payload) = serde_json::from_str::<Vec<String>>(event.payload()) {
                    tauri::async_runtime::spawn(uninstall_packages(payload));
                }
            });
            app.listen("disable", |event| {
                if let Ok(payload) = serde_json::from_str::<Vec<String>>(event.payload()) {
                    tauri::async_runtime::spawn(disable_packages(payload));
                }
            });
            app.listen("revert", |event| {
                if let Ok(payload) = serde_json::from_str::<Vec<String>>(event.payload()) {
                    tauri::async_runtime::spawn(revert_packages(payload));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![scan, list_packages])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
