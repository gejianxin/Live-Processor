use anyhow::Result;
use std::fs;
use std::path::Path;
#[cfg(windows)]
use std::process::Command;

use crate::config::{get_start_menu_programs_dir, is_64bit_os};

#[allow(unused_variables)]
pub fn create_shortcut(
    shortcut_path: &Path,
    target_path: &Path,
    arguments: Option<&str>,
    description: Option<&str>,
    icon_path: Option<&Path>,
) -> Result<()> {
    #[cfg(windows)]
    {
        let script = format!(
            r#"$ws = New-Object -ComObject WScript.Shell;
$s = $ws.CreateShortcut('{}');
$s.TargetPath = '{}';
$s.Arguments = '{}';
$s.Description = '{}';
{}
$s.Save();
"#,
            shortcut_path.display().to_string().replace('\'', "''"),
            target_path.display().to_string().replace('\'', "''"),
            arguments.unwrap_or("").replace('\'', "''"),
            description.unwrap_or("").replace('\'', "''"),
            icon_path
                .map(|p| format!(
                    "$s.IconLocation = '{}';",
                    p.display().to_string().replace('\'', "''")
                ))
                .unwrap_or_default()
        );

        let _ = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status();
    }

    #[cfg(not(windows))]
    {
        let desktop_content = format!(
            "[Desktop Entry]\nType=Application\nName={}\nExec={} {}\nComment={}\nTerminal=false\nCategories=AudioVideo;Audio;\n",
            shortcut_path.file_stem().and_then(|s| s.to_str()).unwrap_or("Live-Processor"),
            target_path.display(),
            arguments.unwrap_or(""),
            description.unwrap_or("")
        );
        let _ = fs::write(shortcut_path.with_extension("desktop"), desktop_content);
    }

    Ok(())
}

pub fn create_start_menu_shortcuts(app_dir: &Path, group_name: &str) -> Result<()> {
    let is_64bit = is_64bit_os();
    let group_dir = get_start_menu_programs_dir().join(group_name);
    let _ = fs::create_dir_all(&group_dir);

    let main_tool = if is_64bit {
        app_dir.join("x64").join("asiolinktool.exe")
    } else {
        app_dir.join("asiolinktool.exe")
    };

    if main_tool.exists() {
        let lnk = group_dir.join("ASIO Link Pro Tool.lnk");
        let _ = create_shortcut(
            &lnk,
            &main_tool,
            None,
            Some("ASIO Link Pro Configuration Tool"),
            Some(&main_tool),
        );
    }

    if is_64bit {
        let tool32 = app_dir.join("asiolinktool.exe");
        if tool32.exists() {
            let lnk32 = group_dir.join("ASIO Link Pro Tool 32.lnk");
            let _ = create_shortcut(
                &lnk32,
                &tool32,
                None,
                Some("ASIO Link Pro 32-bit Tool"),
                Some(&tool32),
            );
        }
    }

    let pdf = app_dir.join("ASIO Link Technology.pdf");
    if pdf.exists() {
        let lnk_pdf = group_dir.join("ASIO Link Technology.lnk");
        let _ = create_shortcut(
            &lnk_pdf,
            &pdf,
            None,
            Some("ASIO Link Technology Manual"),
            None,
        );
    }

    let uninstaller = app_dir.join("live-processor.exe");
    if uninstaller.exists() {
        let lnk_uninst = group_dir.join("Uninstall Live-Processor.lnk");
        let _ = create_shortcut(
            &lnk_uninst,
            &uninstaller,
            Some("uninstall"),
            Some("Uninstall ASIO Link Pro / Live-Processor"),
            None,
        );
    }

    Ok(())
}

pub fn remove_start_menu_shortcuts(group_name: &str) {
    let group_dir = get_start_menu_programs_dir().join(group_name);
    if group_dir.exists() {
        let _ = fs::remove_dir_all(&group_dir);
    }
}
