# SysMonitor

[![Latest release](https://img.shields.io/github/v/release/saqibzahoor-dev/sysmonitor?label=release&color=00ff41)](https://github.com/saqibzahoor-dev/sysmonitor/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/saqibzahoor-dev/sysmonitor/total?color=00d4ff)](https://github.com/saqibzahoor-dev/sysmonitor/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Windows 10/11](https://img.shields.io/badge/Windows-10%20%7C%2011-blue)](https://github.com/saqibzahoor-dev/sysmonitor/releases/latest)

Real-time PC monitoring widget for Windows. A pinned, always-on-top status bar showing CPU / GPU / RAM / disk / network / processes / IP — plus a full 7-tab dashboard with hardware inventory. Built with Tauri v2 + Rust + Svelte 5.

```
 ┌──────────────────────────────────────────┐
 │ root@sysmon:~$                      ─  x │
 │ [OVR][CPU][MEM][GPU][NET][PROC][SYS]     │
 │──────────────────────────────────────────│
 │ root@sysmon:~$ stat --all                │
 │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
 │   CPU 42% │ RAM 71% │ GPU 12% 47°C       │
 │   ▼ 1.2 MB/s    ▲ 0.1 MB/s    192.168.0.x│
 │   ╭──────────────────────────────╮       │
 │   │  █▄  ██▄   █                │       │
 │   │██  ██   █▄█ ▄██             │       │
 │   ╰──────────────────────────────╯       │
 └──────────────────────────────────────────┘
```

---

## ⬇️ Download

> ### **[Download SysMonitor for Windows (.exe — 3.9 MB)](https://github.com/saqibzahoor-dev/sysmonitor/releases/latest/download/SysMonitor_2.0.0_x64-setup.exe)** ← click to download

**Alternative formats:**

| Format | Size | Direct link |
|---|---|---|
| NSIS installer *(recommended)* | 3.9 MB | [SysMonitor_2.0.0_x64-setup.exe](https://github.com/saqibzahoor-dev/sysmonitor/releases/latest/download/SysMonitor_2.0.0_x64-setup.exe) |
| MSI installer | 5.6 MB | [SysMonitor_2.0.0_x64.msi](https://github.com/saqibzahoor-dev/sysmonitor/releases/latest/download/SysMonitor_2.0.0_x64.msi) |
| All releases / changelog | — | [github.com/saqibzahoor-dev/sysmonitor/releases](https://github.com/saqibzahoor-dev/sysmonitor/releases) |

**System requirements:** Windows 10 or 11 (64-bit), .NET Framework 4.8 (preinstalled on Win10/11). ~25 MB on disk after install.

---

## 📦 How to install

### Step-by-step

1. **Click [Download](https://github.com/saqibzahoor-dev/sysmonitor/releases/latest/download/SysMonitor_2.0.0_x64-setup.exe)** — saves `SysMonitor_2.0.0_x64-setup.exe` to your Downloads folder.

2. **Double-click the installer** in File Explorer.

3. **If you see a SmartScreen warning** ("Windows protected your PC"):
   - Click **"More info"**
   - Click **"Run anyway"**
   - *(This is normal — the installer isn't code-signed yet. You can verify the file by comparing its size to the one listed above.)*

4. **The installer runs automatically** — no UAC prompt, no setup wizard. It extracts to `%LOCALAPPDATA%\SysMonitor\` and exits.

5. **SysMonitor launches.** A thin status bar appears at the **bottom-left** of your primary monitor showing your CPU / RAM / GPU / network in real time.

6. **(Optional) Pin it to startup:** the installer adds a Start Menu entry. To auto-launch at login, right-click the SysMonitor entry in Start Menu → **Open file location** → drag the shortcut into `shell:startup` (run `Win+R` → `shell:startup`).

### To get CPU temperature readings

CPU temperature requires **administrator privileges** for the LibreHardwareMonitor kernel driver to access AMD/Intel ring-0 thermal sensors. GPU temperature works without admin.

1. **Quit** SysMonitor if running (right-click tray → Quit).
2. Open File Explorer → navigate to `%LOCALAPPDATA%\SysMonitor\`
3. **Right-click** `sysmonitor.exe` → **Run as administrator**
4. Accept the UAC prompt
5. CPU temperature now shows in the bar and the CPU tab

To make admin launch persistent, create a shortcut → Properties → Advanced → check **Run as administrator**.

### How to uninstall

- Settings → Apps → **SysMonitor** → Uninstall, **OR**
- Run `%LOCALAPPDATA%\SysMonitor\uninstall.exe`

Uninstall removes everything cleanly (the .exe is per-user so it does not touch other users on the machine).

---

## Features

- **CPU** — usage %, per-core ASCII bars, frequency, temperature *(admin required for AMD/Intel sensors)*
- **Memory** — used/total %, top-5 RAM consumers, swap usage
- **GPU** — load %, temperature, VRAM (NVIDIA / AMD / Intel via LibreHardwareMonitor)
- **Disk** — per-drive read/write throughput, free space, SMART health flag
- **Network** — live throughput with ASCII chart, ping, IP, SSID, gateway, DNS, MAC
- **Processes** — sortable top-10 by CPU/RAM, total count, system uptime
- **System inventory** — motherboard, BIOS, RAM sticks, CPU, GPU models, drives, network adapters
- **Compact bar** — content-fit pinned strip, never auto-hides (immune to Start Menu / Win+D behavior)
- **Full window** — frameless 7-tab dashboard
- **Tray-only mode** — minimize to system tray
- **Always on top**, **frameless**, **draggable**, **sharp edges** (DWM rounded corners disabled)

## Display modes

| Mode | Behavior |
|------|----------|
| **Compact (floating)** *(default)* | Content-sized pinned bar — drag anywhere; stays on top |
| **Compact (AppBar)** | Full screen-edge dock — reserves space, other windows shrink |
| **Full window** | Frameless 7-tab dashboard |
| **Tray-only** | Hidden; only the tray icon remains |

Switch from the right-click tray menu or the compact bar's right-click context menu.

## Full window tabs

| Tab | Content |
|-----|---------|
| **OVR** | At-a-glance 6-cell grid (CPU%, RAM%, GPU%, ↓↑, IP) |
| **CPU** | Per-core usage with ASCII bars, frequency, temperature |
| **MEM** | RAM/swap bars + top-5 RAM consumers |
| **GPU** | Per-GPU load, temperature, VRAM |
| **NET** | Throughput, ping, full network info, ASCII chart |
| **PROC** | Sortable top-10 process table + uptime |
| **SYS** | Hardware inventory (mobo, BIOS, RAM, CPU, GPU, drives, network adapters) |

## Tray icon menu

- **Show full window** — opens the 7-tab dashboard
- **Show compact (AppBar)** — docks above the taskbar with reserved space
- **Show compact (floating)** — floating bar above the taskbar
- **Hide all (tray-only)** — only the tray icon remains
- **Always on top** (toggle)
- **Retry sensors** — restart the hardware sensor sidecar
- **About** / **Quit**

---

## Tech Stack

- **[Tauri v2](https://tauri.app/)** — app shell, multi-window, system tray, IPC
- **[Rust 1.77+](https://www.rust-lang.org/)** — monitoring engine, AppBar integration, Win32 DWM calls
- **[Svelte 5](https://svelte.dev/) + [SvelteKit](https://svelte.dev/docs/kit)** — frontend (two routes: full + compact)
- **[LibreHardwareMonitorLib](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor)** (MPL-2.0) via C# sidecar — CPU/GPU temperatures
- **[sysinfo](https://crates.io/crates/sysinfo)**, **[wmi](https://crates.io/crates/wmi)**, **[windows](https://crates.io/crates/windows)** — Rust crates for OS metrics
- **[JetBrains Mono](https://www.jetbrains.com/lp/mono/)** (OFL-1.1) — bundled monospace font

## Build from source

### Prerequisites

- Rust 1.77+ with MSVC target (`rustup default stable-msvc`)
- .NET SDK 8 with `net48` target (for the C# sidecar)
- Visual Studio 2022 Build Tools (C++ workload + Windows 11 SDK)
- Node.js 18+
- Windows 10/11 (target platform)

### Build & run

```powershell
# Install JS deps
npm install

# Build the C# sidecar binary (one-time / after sidecar changes)
./scripts/build-sidecar.ps1

# Dev mode (hot reload)
cargo tauri dev

# Production build (installer + portable exe)
cargo tauri build
```

### Tests

```powershell
cd src-tauri
cargo test
# 114 tests pass
```

## Project Structure

```
src-tauri/src/
  lib.rs                # Orchestrator, 1s tick loop, system-update event
  cpu_monitor.rs        # Per-core CPU usage + frequency (sysinfo)
  cpu_temp.rs           # CPU temperature (WMI thermal zone fallback)
  memory_monitor.rs     # RAM + swap (sysinfo)
  disk_monitor.rs       # Per-drive I/O + free space (sysinfo + WMI)
  process_monitor.rs    # Top-N processes + uptime (sysinfo)
  gpu_monitor.rs        # GPU data from sidecar bridge
  network_monitor.rs    # Byte-counter delta for net throughput
  ping_monitor.rs       # ICMP ping to 8.8.8.8
  network_info.rs       # SSID, IPs, gateway, DNS, MAC, signal
  session_tracker.rs    # Totals, peaks, active connections
  event_logger.rs       # Speed drop / IP change detection
  system_info.rs        # One-shot WMI hardware inventory
  lhm_bridge.rs         # Sidecar spawn + JSON parse (tokio-based)
  appbar.rs             # SHAppBarMessage register/unregister + DWM corner pref
  corner_position.rs    # Pure corner-position math (TDD-covered)
  compact_menu.rs       # Native popup-menu layout
  settings.rs           # Persistent settings + v1 migration
  tray.rs               # System tray icon & menu
sidecar/
  Program.cs            # LibreHardwareMonitorLib wrapper (net48 + Costura)
  SysmonSensor.csproj
src/
  routes/
    +page.svelte        # Full window (7 tabs)
    compact/+page.svelte # Compact bar route
  lib/components/       # OverviewTab, CpuTab, MemTab, GpuTab, NetTab,
                        # ProcTab, SysTab, CompactBar, TitleBar, TabBar, ContextMenu
  lib/stores/system.js  # Svelte store + Tauri event listener
  lib/utils/formatting.js
  styles/               # Terminal theme CSS
```

---

## Troubleshooting

- **"Windows protected your PC" SmartScreen warning** — normal for unsigned installers. Click "More info" → "Run anyway". Source is on GitHub; you can build from source if you prefer.
- **CPU temperature shows `—`** — you're running non-elevated. Right-click `%LOCALAPPDATA%\SysMonitor\sysmonitor.exe` → Run as administrator.
- **GPU temperature shows `—`** — your GPU may not be supported by LibreHardwareMonitor. Currently supports NVIDIA, AMD (discrete + most APUs), and Intel.
- **Bar disappeared** — left-click the SysMonitor tray icon to bring it back, or right-click → Show compact (floating).
- **Bar in wrong corner** — right-click the bar → choose a position preset, or just drag it.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT (this project) — see [LICENSE](LICENSE). LibreHardwareMonitorLib is MPL-2.0; the bundled DLL ships with its own license notice in the installer.

---

<sub>Built by [Saqib Zahoor](https://github.com/saqibzahoor-dev) · [WeboTech Studio](https://webotechstudio.com)</sub>
