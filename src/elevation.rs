use anyhow::{bail, Result};
use std::process::Command;

pub fn is_elevated() -> bool {
    // Check administrative privilege using 'net session' or whoami /groups
    let output = Command::new("net")
        .arg("session")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match output {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

pub fn require_elevation() -> Result<()> {
    if !is_elevated() {
        bail!("This operation requires Administrator privileges. Please re-run Live-Processor as Administrator (Run as administrator).");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn relaunch_elevated() -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Use PowerShell Start-Process with -Verb RunAs to request elevation prompt
    let mut ps_args = format!("Start-Process -FilePath '{}' ", current_exe.display());
    if !args.is_empty() {
        let joined_args = args
            .iter()
            .map(|a| format!("'{}'", a.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ");
        ps_args.push_str(&format!("-ArgumentList {} ", joined_args));
    }
    ps_args.push_str("-Verb RunAs");

    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&ps_args)
        .status()?;

    if status.success() {
        std::process::exit(0);
    } else {
        bail!("Failed to request elevation. Please manually run as Administrator.");
    }
}
