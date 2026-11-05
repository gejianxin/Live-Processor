use anyhow::Result;
use std::path::Path;

#[cfg(windows)]
use anyhow::{bail, Context};
#[cfg(windows)]
use std::process::Command;
#[cfg(windows)]
use crate::config::is_64bit_os;

#[cfg(windows)]
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
    #[cfg(windows)]
    {
        let is_64bit = is_64bit_os();
        let (regsvr32_32, regsvr32_64) = get_regsvr32_paths();

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
    }

    #[cfg(not(windows))]
    {
        // If Wine is available on Linux, attempt registration
        if let Ok(wine) = which_wine() {
            let dll32 = app_dir.join("asiolink.dll");
            if dll32.exists() {
                let _ = std::process::Command::new(wine)
                    .args(["regsvr32", "/s", dll32.to_str().unwrap_or("")])
                    .status();
            }
        }
    }

    Ok(())
}

pub fn unregister_com_libraries(app_dir: &Path) -> Result<()> {
    #[cfg(windows)]
    {
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
    }

    #[cfg(not(windows))]
    {
        let _ = app_dir;
    }

    Ok(())
}

#[cfg(not(windows))]
fn which_wine() -> Result<std::path::PathBuf> {
    for p in &["/usr/bin/wine", "/usr/local/bin/wine"] {
        let pb = std::path::PathBuf::from(p);
        if pb.exists() {
            return Ok(pb);
        }
    }
    anyhow::bail!("wine not found");
}
