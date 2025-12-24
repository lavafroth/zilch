use crate::ShellRunError;
use adb_client::{ADBDeviceExt, ADBUSBDevice};

pub trait ShellCommandText {
    fn shell_command_text(&mut self, command: &str) -> Result<String, ShellRunError>;
}

impl ShellCommandText for ADBUSBDevice {
    fn shell_command_text(&mut self, command: &str) -> Result<String, ShellRunError> {
        let mut buf = Vec::with_capacity(4096);
        self.shell_command(&[command], &mut buf)
            .map_err(|e| match e {
                adb_client::RustADBError::UsbError(rusb::Error::Timeout) => ShellRunError::Timeout,
                _ => ShellRunError::Unrecoverable,
            })?;
        String::from_utf8(buf).map_err(|_| ShellRunError::ParseError)
    }
}
