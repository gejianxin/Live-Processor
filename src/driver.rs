use anyhow::Result;
use std::path::Path;

#[cfg(windows)]
use anyhow::{bail, Context};
#[cfg(windows)]
use crate::config::{is_64bit_os, silent_command};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum DriverInstallResult {
    Success,
    RebootRequired,
    AlreadyInstalled,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum DriverUninstallResult {
    Success,
    RebootRequired,
    ManualRemovalRequired,
}

#[allow(dead_code)]
pub fn get_installer_exe_path(app_dir: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
    #[cfg(windows)]
    {
        if is_64bit_os() {
            let dir = app_dir.join("x64");
            let exe = dir.join("installer.exe");
            (exe, dir)
        } else {
            let exe = app_dir.join("installer.exe");
            (exe, app_dir.to_path_buf())
        }
    }

    #[cfg(not(windows))]
    {
        let exe = app_dir.join("installer.exe");
        (exe, app_dir.to_path_buf())
    }
}

pub fn install_driver(app_dir: &Path) -> Result<DriverInstallResult> {
    #[cfg(windows)]
    {
        let (installer_exe, working_dir) = get_installer_exe_path(app_dir);

        if !installer_exe.exists() {
            bail!(
                "Driver installer executable not found at: {}",
                installer_exe.display()
            );
        }

        let status = silent_command(&installer_exe)
            .arg("install")
            .current_dir(&working_dir)
            .status()
            .with_context(|| format!("Failed to execute {}", installer_exe.display()))?;

        let code = status.code().unwrap_or(-1);
        match code {
            0 => Ok(DriverInstallResult::Success),
            2 => Ok(DriverInstallResult::RebootRequired),
            3 => Ok(DriverInstallResult::AlreadyInstalled),
            1 => bail!("Driver installer reported failure (exit code 1)."),
            other => bail!("Driver installer exited with unexpected error code: {}", other),
        }
    }

    #[cfg(not(windows))]
    {
        let _ = app_dir;
        Ok(DriverInstallResult::Success)
    }
}

pub fn uninstall_driver(app_dir: &Path) -> Result<DriverUninstallResult> {
    #[cfg(windows)]
    {
        let (installer_exe, working_dir) = get_installer_exe_path(app_dir);

        if !installer_exe.exists() {
            return Ok(DriverUninstallResult::Success);
        }

        let status = silent_command(&installer_exe)
            .arg("uninstall")
            .current_dir(&working_dir)
            .status()
            .with_context(|| format!("Failed to execute {}", installer_exe.display()))?;

        let code = status.code().unwrap_or(-1);
        match code {
            0 => Ok(DriverUninstallResult::Success),
            2 => Ok(DriverUninstallResult::RebootRequired),
            1 => Ok(DriverUninstallResult::ManualRemovalRequired),
            _ => Ok(DriverUninstallResult::Success),
        }
    }

    #[cfg(not(windows))]
    {
        let _ = app_dir;
        Ok(DriverUninstallResult::Success)
    }
}

pub fn query_driver_service_status() -> Option<String> {
    #[cfg(windows)]
    {
        let output = silent_command("sc")
            .args(["query", "asiovadpro"])
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("STATE") {
                    return Some(line.trim().to_string());
                }
            }
            Some("INSTALLED".to_string())
        } else {
            None
        }
    }

    #[cfg(not(windows))]
    {
        // On Linux, report active audio servers
        let mut servers = Vec::new();
        if std::process::Command::new("pgrep").arg("pipewire").output().map(|o| o.status.success()).unwrap_or(false) {
            servers.push("PipeWire");
        }
        if std::process::Command::new("pgrep").arg("pulseaudio").output().map(|o| o.status.success()).unwrap_or(false) {
            servers.push("PulseAudio");
        }
        if std::process::Command::new("pgrep").arg("jackd").output().map(|o| o.status.success()).unwrap_or(false) {
            servers.push("JACK");
        }

        if !servers.is_empty() {
            Some(format!("Linux Audio Host: {}", servers.join(", ")))
        } else {
            Some("Linux ALSA Subsystem".to_string())
        }
    }
}
