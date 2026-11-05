use std::path::PathBuf;
use clap::ValueEnum;

#[allow(dead_code)]
pub const DEFAULT_APP_DIR_NAME: &str = "ASIOLinkPro";
#[allow(dead_code)]
pub const DEFAULT_START_MENU_GROUP: &str = "ASIO Link Pro";
#[allow(dead_code)]
pub const REGISTRY_FIX_TASK_NAME: &str = "ASIOLinkPro Registry Fix";

#[allow(dead_code)]
pub const REG_KEY_ASIOLINK: &str = "Software\\ASIOLinkPro";
#[allow(dead_code)]
pub const REG_KEY_ODEUSAUDIO: &str = "SOFTWARE\\ODeusAudio\\ASIOLinkPro";
#[allow(dead_code)]
pub const REG_KEY_CI_POLICY: &str = "SYSTEM\\CurrentControlSet\\Control\\CI\\Policy";
#[allow(dead_code)]
pub const REG_KEY_UNINSTALL: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\ASIOLinkPro";
#[allow(dead_code)]
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

    #[allow(dead_code)]
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
    #[cfg(windows)]
    {
        if let Ok(pf32) = std::env::var("ProgramFiles(x86)") {
            PathBuf::from(pf32).join(DEFAULT_APP_DIR_NAME)
        } else if let Ok(pf) = std::env::var("ProgramFiles") {
            PathBuf::from(pf).join(DEFAULT_APP_DIR_NAME)
        } else {
            PathBuf::from("C:\\Program Files (x86)").join(DEFAULT_APP_DIR_NAME)
        }
    }

    #[cfg(not(windows))]
    {
        // On Linux / Unix, check if running in a Wine prefix
        if let Ok(wineprefix) = std::env::var("WINEPREFIX") {
            PathBuf::from(wineprefix)
                .join("drive_c")
                .join("Program Files (x86)")
                .join(DEFAULT_APP_DIR_NAME)
        } else if let Ok(home) = std::env::var("HOME") {
            let wine_default = PathBuf::from(&home)
                .join(".wine")
                .join("drive_c")
                .join("Program Files (x86)")
                .join(DEFAULT_APP_DIR_NAME);
            if wine_default.parent().map(|p| p.exists()).unwrap_or(false) {
                wine_default
            } else {
                PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("live-processor")
            }
        } else {
            PathBuf::from("/opt/live-processor")
        }
    }
}

pub fn is_64bit_os() -> bool {
    #[cfg(windows)]
    {
        std::env::var("ProgramFiles(x86)").is_ok() || cfg!(target_arch = "x86_64") || cfg!(target_arch = "aarch64")
    }

    #[cfg(not(windows))]
    {
        cfg!(target_pointer_width = "64")
    }
}

pub fn get_system_drivers_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(sys_root) = std::env::var("SystemRoot") {
            PathBuf::from(sys_root).join("System32").join("drivers")
        } else {
            PathBuf::from("C:\\Windows\\System32\\drivers")
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(wineprefix) = std::env::var("WINEPREFIX") {
            PathBuf::from(wineprefix)
                .join("drive_c")
                .join("windows")
                .join("system32")
                .join("drivers")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".wine")
                .join("drive_c")
                .join("windows")
                .join("system32")
                .join("drivers")
        } else {
            std::env::temp_dir().join("asiovadpro_drivers")
        }
    }
}

pub fn get_start_menu_programs_dir() -> PathBuf {
    #[cfg(windows)]
    {
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

    #[cfg(not(windows))]
    {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".local").join("share").join("applications")
        } else {
            PathBuf::from("/usr/share/applications")
        }
    }
}


/// Spawns a Command without showing a console window on Windows.
pub fn silent_command<P: AsRef<std::ffi::OsStr>>(program: P) -> std::process::Command {
    #[allow(unused_mut)]
    let mut cmd = std::process::Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}
