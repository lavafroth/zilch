use crate::adb_shell_text::ShellCommandText;
use crate::{Package, PackageIdentifier, ShellRunError};
use adb_client::ADBUSBDevice;

pub enum Action {
    Uninstall(Package),
    Revert(PackageIdentifier, bool),
    Disable(PackageIdentifier),
}

impl Action {
    pub fn apply_on_device(self, device: &mut ADBUSBDevice) -> Result<(), ShellRunError> {
        match self {
            Action::Uninstall(pkg) => {
                if pkg.path.is_empty() {
                    return Err(ShellRunError::BackupNotPossible(pkg.id));
                }

                let _copy_command_no_output = device.shell_command_text(&format!(
                    "cp {} /data/local/tmp/{}.apk",
                    pkg.path, pkg.id
                ))?;

                let output =
                    device.shell_command_text(&format!("pm uninstall --user 0 -k {}", pkg.id))?;

                if !output.contains("Success") {
                    return Err(ShellRunError::UninstallFailed(pkg.id));
                }
            }
            Action::Revert(id, was_disabled) => {
                if was_disabled {
                    let revert_command = format!("pm enable {id}");
                    let output = device.shell_command_text(&revert_command)?;
                    if !output.contains("new state: enabled") {
                        return Err(ShellRunError::RevertFailed(id));
                    }
                } else {
                    let revert_command = format!("pm install-existing {id}");
                    let output = device.shell_command_text(&revert_command)?;

                    if !output.contains("inaccessible or not found") {
                        return Ok(());
                    }

                    let revert_command = format!("pm install -r --user 0 /data/local/tmp/{id}.apk");
                    let output = device.shell_command_text(&revert_command)?;
                    if !output.contains("Success") {
                        return Err(ShellRunError::RevertFailed(id));
                    }
                }
            }
            Action::Disable(id) => {
                let disable_command = format!("pm disable-user {id}");
                let output = device.shell_command_text(&disable_command)?;
                if !output.contains("new state: disabled-user") {
                    return Err(ShellRunError::DisableFailed(id));
                }
            }
        }
        Ok(())
    }
}
