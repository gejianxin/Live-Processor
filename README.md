# Live-Processor

[![CI](https://github.com/gejianxin/Live-Processor/actions/workflows/ci.yml/badge.svg)](https://github.com/gejianxin/Live-Processor/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20(x64%20%26%20ARM64)-0078D6.svg)](https://github.com/gejianxin/Live-Processor)

**Live-Processor** is a modern, high-performance, cross-platform Rust manager and supervisor for **ASIO Link Pro (v2.4.4.2)**. Rebuilt from the ground up to replace legacy Inno Setup / NSIS script workflows, it provides autonomous driver provisioning, real-time diagnostics, Code Integrity self-healing, and dynamic audio routing layout switching across **Windows (x64)** and **Linux (x86_64 & ARM64)**.

---

## 🌐 Supported Platforms & Targets

| Platform | Target Architecture | Binary | Primary Role |
| :--- | :--- | :--- | :--- |
| **Windows** | `x86_64-pc-windows-msvc` | `live-processor-windows-x64.exe` | Native Driver Installer, Task Scheduler, DIFxAPI, CI Policy Self-Heal |
| **Linux (x64)** | `x86_64-unknown-linux-gnu` | `live-processor-linux-x64` | Cross-Platform CLI, Wine/Proton Prefix Manager, PipeWire/JACK Diagnostics, Payload Extractor |
| **Linux (ARM64)** | `aarch64-unknown-linux-gnu` | `live-processor-linux-arm64` | Cross-Platform CLI, Wine/Box64/FEX Manager, Diagnostics, Payload Extractor |

---

## 📖 Background & Heritage

* **Original Software**: Created by **John Shield / O Deus Audio (RIP)**, ASIO Link Pro is an iconic Windows virtual audio routing driver enabling low-latency routing between multiple DAWs, WDM applications, and hardware interfaces.
* **Post-Mortem Patch**: Created by **fjlj / Give Academy**, removing trial restrictions and random audio cutouts after the original developer's passing.
* **NSIS / Inno Setup Repack**: Created by **DirkoAudio / ASIOLinkProRepack**, bundling the fixed DLLs and identifying the critical Code Integrity (`CI\Policy\UpgradedSystem`) issue.
* **Live-Processor (This Project)**: Full cross-platform Rust rewrite into a modern, native CLI & interactive wizard with embedded payload, zero-dependency installation, diagnostic reporting, and automated lifecycle management.

---

## ✨ Features

- 🚀 **Zero External Dependencies**: Bundles all 32-bit & 64-bit drivers, patched DLLs, tools, and documentation into a single standalone binary (`live-processor`, ~2.7 MB compressed).
- 🎛️ **Dual Device Layout Support**:
  - `4x8`: 4 WDM devices with 8 channels (Original Default)
  - `16x2`: 16 stereo WDM devices (Multi-endpoint routing)
- 🔄 **Dynamic Layout Switching**: Switch layouts on an active system with a single command (`live-processor switch-layout`) without manual reinstall.
- 🛡️ **Autonomous Self-Heal Fix (Windows)**: Automatically applies `HKLM\SYSTEM\CurrentControlSet\Control\CI\Policy\UpgradedSystem = 1` and registers a Windows Scheduled Task (`ASIOLinkPro Registry Fix`) triggered on boot, logon, logoff (Event 4647), and shutdown (Event 13).
- 🐧 **Linux & Wine / Proton Ready**:
  - Inspects active Linux sound hosts (**PipeWire**, **PulseAudio**, **JACK**, **ALSA**).
  - Configures Wine / Proton prefixes automatically for Windows DAW / VST audio production on Linux.
  - Automatically generates Freedesktop `.desktop` application launchers.
- 🔒 **Locale-Agnostic ACL Permissions**: Grants Built-in Users (`S-1-5-32-545`) Full Control on `asiovadpro.dat` and registry keys across all Windows display languages, and POSIX permissions on Unix.
- 🩺 **Diagnostic & Health Engine**: Run `live-processor status` to check driver service state, registry integrity, audio server health, and CI protection status.
- 💻 **Flexible Modes**: Run interactively (TUI/Wizard mode) or headlessly via scriptable CLI (`--silent`).

---

## 🚀 Quick Start

### Interactive Wizard
Simply run `live-processor` in an elevated terminal (or double-click on Windows):
```powershell
.\live-processor.exe
```

### Command Line Usage

#### 1. System Diagnostics & Health Status
```bash
live-processor status
```

Output preview (Windows):
```
=======================================================
       Live-Processor / ASIO Link Pro Status Report    
=======================================================
Host Platform:          Windows - x86_64 (64-bit)
Installation Directory: C:\Program Files (x86)\ASIOLinkPro [FOUND]
 - ASIO Link Tool:      OK
 - Patched DLL:         OK
Device Configuration:   4 WDM devices / 8 channels (Default) (Devices: 4, Channels: 8)
Driver Binary Data:     Present (C:\WINDOWS\System32\drivers\asiovadpro.dat)
Audio Driver / Host:    Active (STATE: 4 RUNNING)
CI Policy (UpgradedSystem): Active (1) [PROTECTED]
Self-Heal Scheduled Task:   Active & Verified [AUTONOMOUS]
-------------------------------------------------------
Ready for cross-platform processing and deployment.
=======================================================
```

Output preview (Linux):
```
=======================================================
       Live-Processor / ASIO Link Pro Status Report    
=======================================================
Host Platform:          Linux - x86_64 (64-bit)
Installation Directory: /home/user/.local/share/live-processor [FOUND]
Device Configuration:   4 WDM devices / 8 channels (Default)
Audio Driver / Host:    Active (Linux Audio Host: PipeWire)
System Integrity:       Native Linux kernel (Unrestricted Code Integrity)
Wine / Proton Support:  Detected (Ready for Windows audio / VST / DAW routing)
-------------------------------------------------------
Ready for cross-platform processing and deployment.
=======================================================
```

#### 2. Install / Upgrade
```bash
# Default installation (4 devices / 8 channels)
live-processor install

# Install with 16 stereo WDM devices
live-processor install --layout 16x2

# Custom directory and silent install
live-processor install --dir "/custom/path" --silent
```

#### 3. Quick Repair (Code Integrity & Scheduled Task)
```bash
live-processor fix
```

#### 4. Switch Device Layout
```bash
live-processor switch-layout --layout 16x2
```

#### 5. Uninstall
```bash
# Standard uninstall (preserves settings)
live-processor uninstall

# Clean uninstall (removes all registry settings and driver data)
live-processor uninstall --clean
```

#### 6. Extract Embedded Payload
```bash
live-processor extract-payload --out ./installer_payload
```

---

## 🛠️ Building from Source

### Native Build
```bash
cargo build --release
```

### Cross-Compiling for Linux
```bash
# Build for Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# Build for Linux ARM64 (aarch64)
cargo build --release --target aarch64-unknown-linux-gnu
```

GitHub Actions will also build and upload release packages for all three architectures automatically upon every commit or version tag.

---

## 📂 Project Structure

```
Live-Processor/
├── .github/workflows/       # Multi-platform CI (Win x64, Linux x64, Linux ARM64) & Release workflows
├── assets/                  # Icons and visual assets
├── docs/                    # Technical architecture & flow docs
├── installer_payload/       # Normalized driver binaries and patched DLLs
│   ├── common/              # Architecture-independent and runtime helpers
│   ├── fixed/               # Patched asiolink.dll (Give Academy)
│   └── variants/            # 4x8 and 16x2 driver packages
├── src/
│   ├── main.rs              # Application entry point & dispatcher
│   ├── cli.rs               # Clap CLI definitions
│   ├── config.rs            # Cross-platform path models & device layouts
│   ├── com.rs               # COM DLL registration (Windows regsvr32 / Wine)
│   ├── driver.rs            # DIFxAPI driver installer & Linux audio host detector
│   ├── elevation.rs         # Windows UAC & Linux sudo elevation checks
│   ├── installer.rs         # Installation orchestrator
│   ├── payload.rs           # Embedded archive unpacker & extractor
│   ├── registry.rs          # Cross-platform registry & Wine prefix configuration
│   ├── security.rs          # SID-based ACL & POSIX permission setter
│   ├── shortcuts.rs         # Windows .lnk & Linux .desktop generator
│   ├── status.rs            # Multi-platform diagnostic reporting
│   ├── task.rs              # Windows Task Scheduler XML & manager
│   ├── uninstaller.rs       # Uninstallation orchestrator
│   └── wizard.rs            # Cross-platform interactive terminal wizard
├── build.rs                 # Compile-time payload packer
├── Cargo.toml               # Rust package manifest
├── NOTICE.md                # Attribution and redistribution notes
└── README.md                # Project documentation
```

---

## ⚖️ Redistribution & Notice

This repository contains recovered binary materials and community patches. Please refer to [`NOTICE.md`](NOTICE.md) for full licensing, attribution, and original EULA notes.
