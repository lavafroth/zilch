use retry::{delay::Fixed, retry};
use std::sync::{Mutex, OnceLock};
use tauri::Emitter;

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

#[tauri::command]
async fn list_packages(app: tauri::AppHandle) -> Result<(), String> {
    let mut dev = DEV
        .0
        .get()
        .expect("could not unwrap handle after initialization; something terrible has happened")
        .lock()
        .expect("could not unwrap handle after initialization; something terrible has happened");
    let pkgs: Vec<String> = dev
        .shell("pm list packages")
        .map_err(|e| e.to_string())?
        .lines()
        .map(|line| line.strip_prefix("package:").unwrap_or(line).to_string())
        .collect();
    app.emit("packages-updated", pkgs)
        .map_err(|_| "failed to send updated package list to the frontend".to_string())?;
    Ok(())
}

pub struct DeviceLock(OnceLock<Mutex<ADBUSBDevice>>);

impl DeviceLock {
    pub fn scan(&self) -> Result<(), String> {
        loop {
            let (vid, pid) = retry(
                Fixed::from_millis(1000),
                || match adb_client::search_adb_devices() {
                    Some(n) => Ok(n),
                    None => Err(()),
                },
            )
            .map_err(|_| "received unit error in function that retries connection".to_string())?;
            let mut device = ADBUSBDevice::new(vid, pid, None).map_err(|e| e.to_string())?;
            device.send_connect().map_err(|e| e.to_string())?;
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
