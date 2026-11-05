use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

use crate::config::is_64bit_os;

pub fn get_regsvr32_paths() -> (String, Option<String>) {
    let win_dir = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    if is_64bit_os() {
        let syswow64 = format!("{}\\SysWOW64\\regsvr32.exe", win_dir);
        let system32 = format!("{}\\System32\\regsvr32.exe", win_dir);
        (syswow64, Some(system32))
    } else {
        let system32 = format!("{}\\System32\\regsvr32.exe", win_dir);
        (system32, None)
    }
}

pub fn register_com_libraries(app_dir: &Path) -> Result<()> {
    let is_64bit = is_64bit_os();
    let (regsvr32_32, regsvr32_64) = get_regsvr32_paths();

    // 1. On 64-bit, register 64-bit DLL first
    if is_64bit {
        if let Some(regsvr64) = regsvr32_64 {
            let dll64 = app_dir.join("x64").join("asiolink.dll");
            if dll64.exists() {
                let status = Command::new(&regsvr64)
                    .args(["/s", dll64.to_str().unwrap_or("")])
                    .status()
                    .with_context(|| format!("Failed to run regsvr32 for {}", dll64.display()))?;

                if !status.success() {
                    bail!("Failed to register 64-bit asiolink.dll");
                }
            }
        }
    }

    // 2. Register 32-bit DLL
    let dll32 = app_dir.join("asiolink.dll");
    if dll32.exists() {
        let status = Command::new(&regsvr32_32)
            .args(["/s", dll32.to_str().unwrap_or("")])
            .status()
            .with_context(|| format!("Failed to run regsvr32 for {}", dll32.display()))?;

        if !status.success() {
            bail!("Failed to register 32-bit asiolink.dll");
        }
    }

    Ok(())
}

pub fn unregister_com_libraries(app_dir: &Path) -> Result<()> {
    let is_64bit = is_64bit_os();
    let (regsvr32_32, regsvr32_64) = get_regsvr32_paths();

    if is_64bit {
        if let Some(regsvr64) = regsvr32_64 {
            let dll64 = app_dir.join("x64").join("asiolink.dll");
            if dll64.exists() {
                let _ = Command::new(&regsvr64)
                    .args(["/s", "/u", dll64.to_str().unwrap_or("")])
                    .status();
            }
        }
    }

    let dll32 = app_dir.join("asiolink.dll");
    if dll32.exists() {
        let _ = Command::new(&regsvr32_32)
            .args(["/s", "/u", dll32.to_str().unwrap_or("")])
            .status();
    }

    Ok(())
}
