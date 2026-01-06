use crate::adb_shell_text::ShellCommandText;
use crate::{Package, PackageIdentifier, ShellRunError};
use adb_client::ADBUSBDevice;

pub enum Action {
    Uninstall(Package),
    Revert(PackageIdentifier, crate::listview::State),
    Disable(PackageIdentifier),
}

impl Action {
    pub fn apply_on_device(
        self,
        device: &mut ADBUSBDevice,
        device_version: u16,
    ) -> Result<(), ShellRunError> {
        match self {
            Action::Uninstall(pkg) => {
                if pkg.path.is_empty() {
                    return Err(ShellRunError::BackupNotPossible(pkg.id));
                }

                let _copy_command_no_output = device.shell_command_text(&format!(
                    "cp {} /data/local/tmp/{}.apk",
                    pkg.path, pkg.id
                ))?;

                let uninstall_command = if device_version < 20 {
                    format!("pm block --user 0 {}", pkg.id)
                } else {
                    format!("pm uninstall --user 0 -k {}", pkg.id)
                };

                let output = device.shell_command_text(&uninstall_command)?;

                if !output.contains("Success") {
                    return Err(ShellRunError::UninstallFailed(pkg.id));
                }
            }
            Action::Revert(id, crate::listview::State::Disabled) => {
                let revert_command = if device_version < 20 {
                    format!("pm unblock --user 0 {}", id)
                } else {
                    format!("pm enable {}", id)
                };

                let output = device.shell_command_text(&revert_command)?;
                if !output.contains("new state: enabled") {
                    return Err(ShellRunError::RevertFailed(id));
                }
            }
            Action::Revert(id, _uninstalled) => {
                let revert_command = if device_version < 20 {
                    format!("pm unblock --user 0 {}", id)
                } else {
                    format!("pm install-existing {id}")
                };
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
            Action::Disable(id) => {
                let disable_command = if device_version < 20 {
                    format!("pm block --user 0 {}", id)
                } else {
                    format!("pm disable-user {id}")
                };
                let output = device.shell_command_text(&disable_command)?;
                if !output.contains("new state: disabled-user") {
                    return Err(ShellRunError::DisableFailed(id));
                }
            }
        }
        Ok(())
    }
}
