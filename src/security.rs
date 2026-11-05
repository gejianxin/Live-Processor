use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Grant Builtin Users (S-1-5-32-545) Full Control on a file path using icacls
pub fn grant_users_full_access_file(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let status = Command::new("icacls")
        .arg(path.as_os_str())
        .args(["/grant", "*S-1-5-32-545:(F)"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .with_context(|| format!("Failed to run icacls on {}", path.display()))?;

    if !status.success() {
        // Fallback: retry with standard English name 'Users' if SID syntax is unsupported on older OS
        let _ = Command::new("icacls")
            .arg(path.as_os_str())
            .args(["/grant", "Users:(F)"])
            .status();
    }

    Ok(())
}

/// Grant Builtin Users (S-1-5-32-545) Full Control on ODeusAudio registry keys
pub fn grant_users_full_access_registry() -> Result<()> {
    let script = r#"
$keys = @("HKLM:\SOFTWARE\ODeusAudio\ASIOLinkPro");
if ([Environment]::Is64BitOperatingSystem) {
    $keys += "HKLM:\SOFTWARE\WOW6432Node\ODeusAudio\ASIOLinkPro";
}
$sid = New-Object System.Security.Principal.SecurityIdentifier("S-1-5-32-545");
$rule = New-Object System.Security.AccessControl.RegistryAccessRule($sid, "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow");

foreach ($k in $keys) {
    if (Test-Path $k) {
        try {
            $acl = Get-Acl $k;
            $acl.SetAccessRule($rule);
            Set-Acl $k $acl;
        } catch {
            # Continue on individual key error
        }
    }
}
"#;

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    Ok(())
}
