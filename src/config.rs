use std::path::PathBuf;
use clap::ValueEnum;

pub const DEFAULT_APP_DIR_NAME: &str = "ASIOLinkPro";
pub const DEFAULT_START_MENU_GROUP: &str = "ASIO Link Pro";
pub const REGISTRY_FIX_TASK_NAME: &str = "ASIOLinkPro Registry Fix";

pub const REG_KEY_ASIOLINK: &str = "Software\\ASIOLinkPro";
pub const REG_KEY_ODEUSAUDIO: &str = "SOFTWARE\\ODeusAudio\\ASIOLinkPro";
pub const REG_KEY_CI_POLICY: &str = "SYSTEM\\CurrentControlSet\\Control\\CI\\Policy";
pub const REG_KEY_UNINSTALL: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\ASIOLinkPro";
pub const REG_KEY_UNINSTALL_LIVE_PROCESSOR: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Live-Processor";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum DeviceLayout {
    /// 4 WDM devices with 8 channels (Original Default)
    #[default]
    #[value(name = "4x8", alias = "default")]
    Layout4x8,
    /// 16 stereo WDM devices (16 devices with 2 channels)
    #[value(name = "16x2", alias = "16stereo")]
    Layout16x2,
}

impl DeviceLayout {
    pub fn num_devices(&self) -> u32 {
        match self {
            Self::Layout4x8 => 4,
            Self::Layout16x2 => 16,
        }
    }

    pub fn num_channels(&self) -> u32 {
        match self {
            Self::Layout4x8 => 8,
            Self::Layout16x2 => 2,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Layout4x8 => "4 devices / 8 channels (Original Default)",
            Self::Layout16x2 => "16 stereo devices / 2 channels (Multi-route)",
        }
    }

    pub fn variant_folder_name(&self) -> &'static str {
        match self {
            Self::Layout4x8 => "4devices_8channels",
            Self::Layout16x2 => "16devices_2channels",
        }
    }
}

pub fn get_default_install_dir() -> PathBuf {
    // Prefer Program Files (x86) on 64-bit systems, or Program Files on 32-bit systems
    if let Ok(pf32) = std::env::var("ProgramFiles(x86)") {
        PathBuf::from(pf32).join(DEFAULT_APP_DIR_NAME)
    } else if let Ok(pf) = std::env::var("ProgramFiles") {
        PathBuf::from(pf).join(DEFAULT_APP_DIR_NAME)
    } else {
        PathBuf::from("C:\\Program Files (x86)").join(DEFAULT_APP_DIR_NAME)
    }
}

pub fn is_64bit_os() -> bool {
    // Standard Windows check
    std::env::var("ProgramFiles(x86)").is_ok() || cfg!(target_arch = "x86_64")
}

pub fn get_system_drivers_dir() -> PathBuf {
    if let Ok(sys_root) = std::env::var("SystemRoot") {
        PathBuf::from(sys_root).join("System32").join("drivers")
    } else {
        PathBuf::from("C:\\Windows\\System32\\drivers")
    }
}

pub fn get_start_menu_programs_dir() -> PathBuf {
    if let Ok(program_data) = std::env::var("ProgramData") {
        PathBuf::from(program_data)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
    } else {
        PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs")
    }
}
