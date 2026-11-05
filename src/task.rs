use anyhow::{bail, Context, Result};
use std::fs;
use std::process::Command;

use crate::config::REGISTRY_FIX_TASK_NAME;

pub fn generate_task_xml() -> String {
    let reg_exe = if let Ok(sys_root) = std::env::var("SystemRoot") {
        format!("{}\\System32\\reg.exe", sys_root)
    } else {
        "C:\\Windows\\System32\\reg.exe".to_string()
    };

    let reg_args = "add \"HKLM\\SYSTEM\\CurrentControlSet\\Control\\CI\\Policy\" /v UpgradedSystem /t REG_DWORD /d 1 /f";

    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Author>ASIOLinkPro Installer</Author>
    <Description>Reapplies the Code Integrity policy registry fix at startup, logon, logoff, and shutdown.</Description>
  </RegistrationInfo>
  <Triggers>
    <BootTrigger>
      <Enabled>true</Enabled>
    </BootTrigger>
    <LogonTrigger>
      <Enabled>true</Enabled>
    </LogonTrigger>
    <EventTrigger>
      <Enabled>true</Enabled>
      <Subscription><![CDATA[<QueryList><Query Id="0" Path="Security"><Select Path="Security">*[System[Provider[@Name='Microsoft-Windows-Security-Auditing'] and EventID=4647]]</Select></Query></QueryList>]]></Subscription>
    </EventTrigger>
    <EventTrigger>
      <Enabled>true</Enabled>
      <Subscription><![CDATA[<QueryList><Query Id="0" Path="System"><Select Path="System">*[System[Provider[@Name='Microsoft-Windows-Kernel-General'] and EventID=13]]</Select></Query></QueryList>]]></Subscription>
    </EventTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>S-1-5-18</UserId>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>false</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <IdleSettings>
      <StopOnIdleEnd>false</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>true</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT5M</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{}</Command>
      <Arguments>{}</Arguments>
    </Exec>
  </Actions>
</Task>"#,
        reg_exe, reg_args
    )
}

pub fn install_registry_fix_task() -> Result<()> {
    let temp_dir = std::env::temp_dir();
    let xml_path = temp_dir.join("asiolinkpro-registryfix-task.xml");

    // Write UTF-16LE or UTF-8 XML (Windows schtasks parses UTF-8 with BOM or standard XML)
    fs::write(&xml_path, generate_task_xml())
        .with_context(|| format!("Failed to write task XML to {}", xml_path.display()))?;

    let output = Command::new("schtasks")
        .args([
            "/Create",
            "/TN",
            REGISTRY_FIX_TASK_NAME,
            "/XML",
            xml_path.to_str().unwrap_or(""),
            "/F",
        ])
        .output()
        .context("Failed to invoke schtasks.exe")?;

    let _ = fs::remove_file(&xml_path);

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to register scheduled task: {}", err_msg.trim());
    }

    Ok(())
}

pub fn verify_registry_fix_task() -> bool {
    let output = Command::new("schtasks")
        .args(["/Query", "/TN", REGISTRY_FIX_TASK_NAME])
        .output();

    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

pub fn remove_registry_fix_task() -> Result<()> {
    let _ = Command::new("schtasks")
        .args(["/Delete", "/TN", REGISTRY_FIX_TASK_NAME, "/F"])
        .output();
    Ok(())
}
