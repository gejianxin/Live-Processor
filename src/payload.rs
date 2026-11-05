use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::fs;
use std::path::{Path, PathBuf};
use tar::Archive;

use crate::config::{get_system_drivers_dir, is_64bit_os, DeviceLayout};

pub static EMBEDDED_PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.tar.gz"));

pub struct PayloadSource {
    base_dir: Option<PathBuf>,
}

impl PayloadSource {
    pub fn new(custom_path: Option<&Path>) -> Result<Self> {
        if let Some(p) = custom_path {
            if p.exists() {
                return Ok(Self {
                    base_dir: Some(p.to_path_buf()),
                });
            }
        }

        // Check if installer_payload exists in current directory or next to executable
        let local_payload = Path::new("installer_payload");
        if local_payload.exists() {
            return Ok(Self {
                base_dir: Some(local_payload.to_path_buf()),
            });
        }

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let exe_payload = parent.join("installer_payload");
                if exe_payload.exists() {
                    return Ok(Self {
                        base_dir: Some(exe_payload),
                    });
                }
            }
        }

        // None means use embedded payload archive
        Ok(Self { base_dir: None })
    }

    /// Extract payload to a temporary directory if using embedded
    pub fn prepare_temp_payload(&self, temp_dir: &Path) -> Result<PathBuf> {
        if let Some(ref dir) = self.base_dir {
            return Ok(dir.clone());
        }

        // Unpack embedded tar.gz into temp_dir
        let gz = GzDecoder::new(EMBEDDED_PAYLOAD);
        let mut archive = Archive::new(gz);
        archive
            .unpack(temp_dir)
            .context("Failed to unpack embedded payload")?;

        let extracted = temp_dir.join("installer_payload");
        if extracted.exists() {
            Ok(extracted)
        } else {
            Ok(temp_dir.to_path_buf())
        }
    }

    /// Install files into target directories based on layout and OS architecture
    pub fn install_files(
        &self,
        app_dir: &Path,
        layout: DeviceLayout,
        temp_extract_dir: &Path,
    ) -> Result<()> {
        let payload_root = self.prepare_temp_payload(temp_extract_dir)?;
        let is_64bit = is_64bit_os();
        let variant_folder = layout.variant_folder_name();

        // 1. Install asiovadpro.dat to %SystemRoot%\System32\drivers
        let sys_drivers_dir = get_system_drivers_dir();
        fs::create_dir_all(&sys_drivers_dir).context("Failed to create system drivers dir")?;

        let src_dat = payload_root.join("common/system-drivers/asiovadpro.dat");
        let dest_dat = sys_drivers_dir.join("asiovadpro.dat");
        if src_dat.exists() {
            fs::copy(&src_dat, &dest_dat)
                .with_context(|| format!("Failed to copy asiovadpro.dat to {}", dest_dat.display()))?;
        }

        // 2. Install application root files
        fs::create_dir_all(app_dir)
            .with_context(|| format!("Failed to create install dir {}", app_dir.display()))?;

        // PDF Documentation
        let src_pdf = payload_root.join("common/root/ASIO Link Technology.pdf");
        if src_pdf.exists() {
            fs::copy(&src_pdf, app_dir.join("ASIO Link Technology.pdf"))?;
        }

        // 32-bit tool
        let src_tool32 = payload_root.join("common/root/asiolinktool.exe");
        if src_tool32.exists() {
            fs::copy(&src_tool32, app_dir.join("asiolinktool.exe"))?;
        }

        // Patched 32-bit DLL
        let src_dll32 = payload_root.join("fixed/root/asiolink.dll");
        if src_dll32.exists() {
            fs::copy(&src_dll32, app_dir.join("asiolink.dll"))?;
        }

        // registryfix.reg
        let src_reg = payload_root.join("common/root/registryfix.reg");
        if src_reg.exists() {
            fs::copy(&src_reg, app_dir.join("registryfix.reg"))?;
        }

        if is_64bit {
            // 64-bit runtime files in {app}\x64
            let x64_dir = app_dir.join("x64");
            fs::create_dir_all(&x64_dir)?;

            // 64-bit tool
            fs::copy(
                payload_root.join("common/x64-runtime/asiolinktool.exe"),
                x64_dir.join("asiolinktool.exe"),
            )?;

            // 64-bit driver installer & difxapi
            fs::copy(
                payload_root.join("common/x64-runtime/installer.exe"),
                x64_dir.join("installer.exe"),
            )?;
            fs::copy(
                payload_root.join("common/x64-runtime/difxapi.dll"),
                x64_dir.join("difxapi.dll"),
            )?;

            // Patched 64-bit DLL
            fs::copy(
                payload_root.join("fixed/x64/asiolink.dll"),
                x64_dir.join("asiolink.dll"),
            )?;

            // Selected driver package variant
            let variant_x64 = payload_root.join("variants").join(variant_folder).join("x64");
            fs::copy(variant_x64.join("asiovadpro.cat"), x64_dir.join("asiovadpro.cat"))?;
            fs::copy(variant_x64.join("asiovadpro.inf"), x64_dir.join("asiovadpro.inf"))?;
            fs::copy(variant_x64.join("asiovadpro.sys"), x64_dir.join("asiovadpro.sys"))?;
        } else {
            // 32-bit runtime files directly in {app}
            fs::copy(
                payload_root.join("common/x86-runtime/installer.exe"),
                app_dir.join("installer.exe"),
            )?;
            fs::copy(
                payload_root.join("common/x86-runtime/difxapi.dll"),
                app_dir.join("difxapi.dll"),
            )?;

            // Selected driver package variant for 32-bit
            let variant_x86 = payload_root.join("variants").join(variant_folder).join("root");
            fs::copy(variant_x86.join("asiovadpro.cat"), app_dir.join("asiovadpro.cat"))?;
            fs::copy(variant_x86.join("asiovadpro.inf"), app_dir.join("asiovadpro.inf"))?;
            fs::copy(variant_x86.join("asiovadpro.sys"), app_dir.join("asiovadpro.sys"))?;
        }

        // Install current live-processor executable into {app} for management and uninstallation
        if let Ok(current_exe) = std::env::current_exe() {
            let app_cli = app_dir.join("live-processor.exe");
            let _ = fs::copy(&current_exe, &app_cli);
        }

        Ok(())
    }

    /// Extract raw payload to user specified directory (for packaging or inspection)
    pub fn extract_all_to(&self, target_dir: &Path) -> Result<()> {
        fs::create_dir_all(target_dir)?;
        let gz = GzDecoder::new(EMBEDDED_PAYLOAD);
        let mut archive = Archive::new(gz);
        archive.unpack(target_dir)?;
        Ok(())
    }
}
