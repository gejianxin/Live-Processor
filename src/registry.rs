use anyhow::{Context, Result};
use std::path::Path;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE, KEY_WOW64_32KEY, KEY_WOW64_64KEY};
use winreg::RegKey;

use crate::config::{
    is_64bit_os, DeviceLayout, REG_KEY_ASIOLINK, REG_KEY_CI_POLICY, REG_KEY_ODEUSAUDIO,
    REG_KEY_UNINSTALL, REG_KEY_UNINSTALL_LIVE_PROCESSOR,
};

pub struct InstalledInfo {
    pub install_dir: Option<String>,
    pub start_menu_folder: Option<String>,
    pub num_devices: Option<u32>,
    pub num_channels: Option<u32>,
    pub ci_policy_applied: bool,
    #[allow(dead_code)]
    pub legacy_uninstall_string: Option<String>,
}

pub fn get_installed_info() -> InstalledInfo {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // Read ASIOLinkPro settings
    let (install_dir, start_menu_folder) = if let Ok(key) = hklm.open_subkey_with_flags(REG_KEY_ASIOLINK, KEY_READ | KEY_WOW64_32KEY) {
        let dir: Option<String> = key.get_value("Install_Dir").ok();
        let folder: Option<String> = key.get_value("Start Menu Folder").ok();
        (dir, folder)
    } else {
        (None, None)
    };

    // Read ODeusAudio device configuration
    let flags = if is_64bit_os() { KEY_READ | KEY_WOW64_64KEY } else { KEY_READ | KEY_WOW64_32KEY };
    let (num_devices, num_channels) = if let Ok(key) = hklm.open_subkey_with_flags(REG_KEY_ODEUSAUDIO, flags) {
        let dev: Option<u32> = key.get_value("NumDevices").ok();
        let ch: Option<u32> = key.get_value("NumChannels").ok();
        (dev, ch)
    } else {
        (None, None)
    };

    // Read Code Integrity policy
    let ci_policy_applied = if let Ok(key) = hklm.open_subkey_with_flags(REG_KEY_CI_POLICY, KEY_READ) {
        key.get_value::<u32, _>("UpgradedSystem").unwrap_or(0) == 1
    } else {
        false
    };

    // Read legacy uninstall string
    let legacy_uninstall_string = if let Ok(key) = hklm.open_subkey_with_flags(REG_KEY_UNINSTALL, KEY_READ | KEY_WOW64_32KEY) {
        key.get_value("UninstallString").ok()
    } else {
        None
    };

    InstalledInfo {
        install_dir,
        start_menu_folder,
        num_devices,
        num_channels,
        ci_policy_applied,
        legacy_uninstall_string,
    }
}

pub fn configure_installation_registry(
    app_dir: &Path,
    start_menu_group: Option<&str>,
    layout: DeviceLayout,
) -> Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // 1. Write HKLM32\Software\ASIOLinkPro
    let (asio_key, _) = hklm
        .create_subkey_with_flags(REG_KEY_ASIOLINK, KEY_WRITE | KEY_WOW64_32KEY)
        .context("Failed to create HKLM\\Software\\ASIOLinkPro key")?;

    asio_key.set_value("Install_Dir", &app_dir.to_string_lossy().to_string())?;
    if let Some(group) = start_menu_group {
        asio_key.set_value("Start Menu Folder", &group)?;
    }

    // 2. Write HKLM\SOFTWARE\ODeusAudio\ASIOLinkPro (both 32-bit and 64-bit views if applicable)
    let views = if is_64bit_os() {
        vec![KEY_WOW64_64KEY, KEY_WOW64_32KEY]
    } else {
        vec![KEY_WOW64_32KEY]
    };

    for view in views {
        if let Ok((odeus_key, _)) = hklm.create_subkey_with_flags(REG_KEY_ODEUSAUDIO, KEY_WRITE | view) {
            let _ = odeus_key.set_value("NumDevices", &layout.num_devices());
            let _ = odeus_key.set_value("NumChannels", &layout.num_channels());
        }
    }

    // 3. Apply Code Integrity Policy: UpgradedSystem = 1
    apply_code_integrity_policy_fix()?;

    // 4. On 32-bit Windows, set DONT_VIRTUALIZE flag via reg.exe
    if !is_64bit_os() {
        let _ = std::process::Command::new("reg.exe")
            .args(["FLAGS", "HKLM\\SOFTWARE\\ODeusAudio\\ASIOLinkPro", "SET", "DONT_VIRTUALIZE", "/s"])
            .status();
    }

    Ok(())
}

pub fn apply_code_integrity_policy_fix() -> Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (ci_key, _) = hklm
        .create_subkey_with_flags(REG_KEY_CI_POLICY, KEY_WRITE)
        .context("Failed to open or create Code Integrity policy key")?;

    let value: u32 = 1;
    ci_key
        .set_value("UpgradedSystem", &value)
        .context("Failed to set UpgradedSystem = 1 in CI policy")?;

    Ok(())
}

pub fn register_uninstall_entry(app_dir: &Path) -> Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let uninstall_exe = app_dir.join("live-processor.exe");
    let display_icon = if is_64bit_os() {
        app_dir.join("x64").join("asiolinktool.exe")
    } else {
        app_dir.join("asiolinktool.exe")
    };

    let entries = [REG_KEY_UNINSTALL_LIVE_PROCESSOR, REG_KEY_UNINSTALL];

    for key_path in entries {
        if let Ok((key, _)) = hklm.create_subkey_with_flags(key_path, KEY_WRITE | KEY_WOW64_32KEY) {
            let _ = key.set_value("DisplayName", &"ASIO Link Pro (Live-Processor Repack)");
            let _ = key.set_value("DisplayVersion", &"2.4.4.2-rev1");
            let _ = key.set_value("Publisher", &"O Deus Audio / Community");
            let _ = key.set_value("InstallLocation", &app_dir.to_string_lossy().to_string());
            let _ = key.set_value("DisplayIcon", &display_icon.to_string_lossy().to_string());
            let _ = key.set_value(
                "UninstallString",
                &format!("\"{}\" uninstall", uninstall_exe.display()),
            );
            let _ = key.set_value(
                "QuietUninstallString",
                &format!("\"{}\" uninstall --silent", uninstall_exe.display()),
            );
            let _ = key.set_value("NoModify", &1u32);
            let _ = key.set_value("NoRepair", &0u32);
        }
    }

    Ok(())
}

pub fn remove_uninstall_entries() {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let _ = hklm.delete_subkey_with_flags(REG_KEY_UNINSTALL_LIVE_PROCESSOR, KEY_WOW64_32KEY);
    let _ = hklm.delete_subkey_with_flags(REG_KEY_UNINSTALL, KEY_WOW64_32KEY);
    let _ = hklm.delete_subkey_with_flags(REG_KEY_ASIOLINK, KEY_WOW64_32KEY);
}

pub fn delete_all_audio_settings() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // Delete HKCU\Software\ODeusAudio\ASIOLinkPro
    let _ = hkcu.delete_subkey_all("Software\\ODeusAudio\\ASIOLinkPro");
    let _ = hkcu.delete_subkey("Software\\ODeusAudio");

    // Remove startup entry
    if let Ok(run_key) = hkcu.open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE) {
        let _ = run_key.delete_value("ASIOLINKPRO");
    }

    // Delete HKLM ODeusAudio
    let _ = hklm.delete_subkey_with_flags(REG_KEY_ODEUSAUDIO, KEY_WOW64_32KEY);
    let _ = hklm.delete_subkey_with_flags("SOFTWARE\\ODeusAudio", KEY_WOW64_32KEY);

    if is_64bit_os() {
        let _ = hklm.delete_subkey_with_flags(REG_KEY_ODEUSAUDIO, KEY_WOW64_64KEY);
        let _ = hklm.delete_subkey_with_flags("SOFTWARE\\ODeusAudio", KEY_WOW64_64KEY);
    }
}
