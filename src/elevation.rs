use anyhow::{bail, Result};
use std::process::Command;

pub fn is_elevated() -> bool {
    #[cfg(windows)]
    {
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

    #[cfg(not(windows))]
    {
        let output = Command::new("id")
            .arg("-u")
            .output();

        match output {
            Ok(out) => {
                let uid = String::from_utf8_lossy(&out.stdout).trim().to_string();
                uid == "0"
            }
            Err(_) => false,
        }
    }
}

pub fn require_elevation() -> Result<()> {
    if !is_elevated() {
        #[cfg(windows)]
        bail!("This operation requires Administrator privileges. Please re-run Live-Processor as Administrator (Run as administrator).");

        #[cfg(not(windows))]
        bail!("This operation requires superuser (root) privileges. Please re-run Live-Processor with sudo.");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn relaunch_elevated() -> Result<()> {
    #[cfg(windows)]
    {
        let current_exe = std::env::current_exe()?;
        let args: Vec<String> = std::env::args().skip(1).collect();

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

    #[cfg(not(windows))]
    {
        let current_exe = std::env::current_exe()?;
        let args: Vec<String> = std::env::args().skip(1).collect();

        let mut cmd = Command::new("sudo");
        cmd.arg(current_exe);
        for a in args {
            cmd.arg(a);
        }

        let status = cmd.status()?;
        if status.success() {
            std::process::exit(0);
        } else {
            bail!("Failed to run with sudo.");
        }
    }
}
