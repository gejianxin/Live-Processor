use std::path::PathBuf;
use crate::config::{get_default_install_dir, is_64bit_os};
use crate::registry::get_installed_info;

/// Bridge and supervisor for ASIO Link Pro kernel driver & helper tools.
pub struct AsioLinkProBridge {
    pub is_driver_active: bool,
    pub is_tool_running: bool,
    pub tool_path: Option<PathBuf>,
}

impl AsioLinkProBridge {
    pub fn probe() -> Self {
        let info = get_installed_info();
        let is_installed = info.install_dir.is_some();

        let tool_path = if let Some(dir) = &info.install_dir {
            let p = PathBuf::from(dir);
            let tool = if is_64bit_os() {
                p.join("x64").join("asiolinktool.exe")
            } else {
                p.join("asiolinktool.exe")
            };
            if tool.exists() { Some(tool) } else { None }
        } else {
            let default_p = get_default_install_dir();
            let tool = if is_64bit_os() {
                default_p.join("x64").join("asiolinktool.exe")
            } else {
                default_p.join("asiolinktool.exe")
            };
            if tool.exists() { Some(tool) } else { None }
        };

        // Check if asiolinktool is actively running
        #[cfg(windows)]
        let is_tool_running = {
            let out = crate::config::silent_command("tasklist")
                .args(["/FI", "IMAGENAME eq asiolinktool.exe", "/NH"])
                .output();
            if let Ok(res) = out {
                let s = String::from_utf8_lossy(&res.stdout);
                s.contains("asiolinktool.exe")
            } else {
                false
            }
        };

        #[cfg(not(windows))]
        let is_tool_running = false;

        Self {
            is_driver_active: is_installed,
            is_tool_running,
            tool_path,
        }
    }

    /// Launch or restart ASIO Link Tool
    pub fn launch_tool(&self) -> bool {
        #[cfg(windows)]
        if let Some(tool) = &self.tool_path {
            if tool.exists() {
                let _ = crate::config::silent_command(tool).spawn();
                return true;
            }
        }
        false
    }
}
