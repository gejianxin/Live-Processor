# Live-Processor

[![CI](https://github.com/gejianxin/Live-Processor/actions/workflows/ci.yml/badge.svg)](https://github.com/gejianxin/Live-Processor/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-0078D6.svg)](https://microsoft.com/windows)

**Live-Processor** is a modern, high-performance, single-binary Rust reimagining and manager for **ASIO Link Pro (v2.4.4.2)**. Rebuilt from the ground up to replace legacy Inno Setup / NSIS script workflows, it provides autonomous driver provisioning, real-time diagnostics, Code Integrity self-healing, and dynamic audio routing layout switching for Windows 10 and Windows 11.

---

## 📖 Background & Heritage

* **Original Software**: Created by **John Shield / O Deus Audio (RIP)**, ASIO Link Pro is an iconic Windows virtual audio routing driver enabling low-latency routing between multiple DAWs, WDM applications, and hardware interfaces.
* **Post-Mortem Patch**: Created by **fjlj / Give Academy**, removing trial restrictions and random audio cutouts after the original developer's passing.
* **NSIS / Inno Setup Repack**: Created by **DirkoAudio / ASIOLinkProRepack**, bundling the fixed DLLs and identifying the critical Code Integrity (`CI\Policy\UpgradedSystem`) issue.
* **Live-Processor (This Project)**: Full Rust rewrite into a modern, native CLI & interactive wizard with embedded payload, zero-dependency installation, diagnostic reporting, and automated lifecycle management.

---

## ✨ Features

- 🚀 **Zero External Dependencies**: Bundles all 32-bit & 64-bit drivers, patched DLLs, tools, and documentation into a single standalone binary (`live-processor.exe`, ~2.7 MB compressed).
- 🎛️ **Dual Device Layout Support**:
  - `4x8`: 4 WDM devices with 8 channels (Original Default)
  - `16x2`: 16 stereo WDM devices (Multi-endpoint routing)
- 🔄 **Dynamic Layout Switching**: Switch layouts on an active system with a single command (`live-processor switch-layout`) without manual reinstall.
- 🛡️ **Autonomous Self-Heal Fix**: Automatically applies `HKLM\SYSTEM\CurrentControlSet\Control\CI\Policy\UpgradedSystem = 1` and registers a Windows Scheduled Task (`ASIOLinkPro Registry Fix`) triggered on boot, logon, logoff (Event 4647), and shutdown (Event 13).
- 🔒 **Locale-Agnostic ACL Permissions**: Grants Built-in Users (`S-1-5-32-545`) Full Control on `asiovadpro.dat` and registry keys across all Windows display languages.
- 🩺 **Diagnostic & Health Engine**: Run `live-processor status` to check driver service state, registry integrity, CI protection status, and task health.
- 💻 **Flexible Modes**: Run interactively (TUI/Wizard mode) or headlessly via scriptable CLI (`--silent`).

---

## 🚀 Quick Start

### Interactive Wizard
Simply run `live-processor.exe` in an elevated command prompt (or double-click):
```powershell
.\live-processor.exe
```

### Command Line Usage

#### 1. Install / Upgrade
```powershell
# Default installation (4 devices / 8 channels)
.\live-processor.exe install

# Install with 16 stereo WDM devices
.\live-processor.exe install --layout 16x2

# Custom directory and silent install
.\live-processor.exe install --dir "C:\Audio\ASIOLinkPro" --silent
```

#### 2. System Diagnostics & Health Status
```powershell
.\live-processor.exe status
```

Output preview:
```
=======================================================
       Live-Processor / ASIO Link Pro Status Report    
=======================================================
System Architecture: 64-bit (x64)
Installation Directory: C:\Program Files (x86)\ASIOLinkPro [FOUND]
 - ASIO Link Tool: OK
 - Patched DLL:    OK
Device Configuration:   4 WDM devices / 8 channels (Default) (Devices: 4, Channels: 8)
System Driver Data:     Present (C:\WINDOWS\System32\drivers\asiovadpro.dat)
Kernel Driver Service:  Registered (STATE: 4 RUNNING)
CI Policy (UpgradedSystem): Active (1) [PROTECTED]
Self-Heal Scheduled Task:   Active & Verified [AUTONOMOUS]
-------------------------------------------------------
All core components are healthy and protected.
=======================================================
```

#### 3. Quick Repair (Code Integrity & Scheduled Task)
```powershell
.\live-processor.exe fix
```

#### 4. Switch Device Layout
```powershell
.\live-processor.exe switch-layout --layout 16x2
```

#### 5. Uninstall
```powershell
# Standard uninstall (preserves settings)
.\live-processor.exe uninstall

# Clean uninstall (removes all registry settings and driver data)
.\live-processor.exe uninstall --clean
```

#### 6. Extract Embedded Payload
```powershell
.\live-processor.exe extract-payload --out ./installer_payload
```

---

## 🛠️ Building from Source

### Prerequisites
- Windows 10 or 11 (x64 / x86)
- [Rust toolchain](https://rustup.rs/) (edition 2021/2024, Rust 1.80+)

### Compile
```powershell
# Clone the repository
git clone https://github.com/gejianxin/Live-Processor.git
cd Live-Processor

# Build release executable
cargo build --release
```

The resulting standalone executable is generated at:
```
target/release/live-processor.exe
```

---

## 📂 Project Structure

```
Live-Processor/
├── .github/workflows/       # GitHub Actions CI & Release automation
├── assets/                  # Icons and visual assets
├── docs/                    # Technical architecture & flow docs
├── installer_payload/       # Normalized driver binaries and patched DLLs
│   ├── common/              # Architecture-independent and runtime helpers
│   ├── fixed/               # Patched asiolink.dll (Give Academy)
│   └── variants/            # 4x8 and 16x2 driver packages
├── src/
│   ├── main.rs              # Application entry point & dispatcher
│   ├── cli.rs               # Clap CLI definitions
│   ├── config.rs            # Configuration constants & device layout models
│   ├── com.rs               # COM DLL registration (regsvr32)
│   ├── driver.rs            # DIFxAPI driver installer & service manager
│   ├── elevation.rs         # Windows UAC elevation checks
│   ├── installer.rs         # Installation orchestrator
│   ├── payload.rs           # Embedded archive unpacker & extractor
│   ├── registry.rs          # Windows registry manipulation
│   ├── security.rs          # SID-based ACL permission setter
│   ├── shortcuts.rs         # Windows Start Menu shortcut generator
│   ├── status.rs            # Diagnostics and reporting
│   ├── task.rs              # Windows Task Scheduler XML & manager
│   ├── uninstaller.rs       # Uninstallation orchestrator
│   └── wizard.rs            # Interactive terminal wizard
├── build.rs                 # Compile-time payload packer
├── Cargo.toml               # Rust package manifest
├── NOTICE.md                # Attribution and redistribution notes
└── README.md                # Project documentation
```

---

## ⚖️ Redistribution & Notice

This repository contains recovered binary materials and community patches. Please refer to [`NOTICE.md`](NOTICE.md) for full licensing, attribution, and original EULA notes.
