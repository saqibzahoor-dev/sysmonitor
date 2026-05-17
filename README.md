# SysMonitor

Real-time PC monitoring widget for Windows with a Kali Linux terminal aesthetic. CPU, RAM, GPU, disk, network, processes, and hardware inventory in a compact bar or full tabbed window.

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

## Features

- **CPU** — usage %, per-core ASCII bars, frequency, temperature (via LibreHardwareMonitor)
- **Memory** — RAM/swap bars, top-5 RAM consumers
- **GPU** — load %, temperature, VRAM (AMD/NVIDIA/Intel via LibreHardwareMonitor)
- **Disk** — per-drive read/write MB/s, free space
- **Network** — download/upload throughput with ASCII chart, ping, IP, SSID, gateway, DNS, MAC
- **Processes** — sortable top-10 by CPU/RAM, system uptime
- **System** — motherboard, BIOS, RAM sticks, CPU, GPU models, drive health (one-shot inventory)
- **Compact bar** — 360×30 px strip docked above the taskbar (Windows AppBar)
- **Full window** — frameless 420×440 px tabbed dashboard
- **Tray-only mode** — minimize to system tray
- **Always on top**, **frameless**, draggable

## Install

Download the latest installer from releases:

- **NSIS:** `SysMonitor_2.0.0_x64-setup.exe` (~3 MB)
- **MSI:** `SysMonitor_2.0.0_x64_en-US.msi` (~4 MB)

Per-user install, **no admin required**. Installs to `%LOCALAPPDATA%\SysMonitor\`.

**Requirements:** Windows 10/11 (x64), .NET Framework 4.8 (preinstalled on Win10/11).

## Modes

| Mode | Description |
|------|-------------|
| **Compact (AppBar)** *(default)* | Thin strip docked above the taskbar — reserves screen space |
| **Compact (floating)** | Same strip but does not reserve space |
| **Full window** | Frameless 7-tab dashboard |
| **Tray-only** | Hidden — only the tray icon remains |

Switch from the right-click tray menu or the right-click context menu on any window.

## Full window tabs

| Tab | Content |
|-----|---------|
| **OVR** | At-a-glance 6-cell grid (CPU%, RAM%, GPU%, ↓↑, IP) |
| **CPU** | Per-core usage with ASCII bars, frequency, temperature |
| **MEM** | RAM/swap bars + top-5 RAM consumers |
| **GPU** | Per-GPU load, temperature, VRAM |
| **NET** | Throughput, ping, full network info, ASCII chart |
| **PROC** | Sortable top-10 process table + uptime |
| **SYS** | Hardware inventory (mobo, BIOS, RAM sticks, CPU, GPU, drives) |

## Tray icon menu

- **Show full window** — opens the 7-tab dashboard
- **Show compact (AppBar)** — docks above the taskbar with reserved space
- **Show compact (floating)** — floating bar above the taskbar
- **Hide all (tray-only)** — only the tray icon remains
- **Always on top** (toggle)
- **Retry sensors** — restart the hardware sensor sidecar (shown when unavailable)
- **About** / **Quit**

## Tech Stack

- [Tauri v2](https://tauri.app/) — app shell, multi-window, system tray, IPC
- [Rust 1.77+](https://www.rust-lang.org/) — monitoring engine, AppBar integration
- [Svelte 5](https://svelte.dev/) + [SvelteKit](https://svelte.dev/docs/kit/introduction) — frontend (SPA, two routes: full + compact)
- [LibreHardwareMonitorLib](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor) (MPL-2.0) via C# sidecar — CPU/GPU temperatures
- [sysinfo](https://crates.io/crates/sysinfo), [wmi](https://crates.io/crates/wmi), [windows](https://crates.io/crates/windows) — Rust crates for OS metrics
- [JetBrains Mono](https://www.jetbrains.com/lp/mono/) (OFL-1.1) — bundled monospace font

## Development

### Prerequisites

- Rust 1.77+ with MSVC target (`rustup default stable-msvc`)
- .NET SDK with `net48` target (for the C# sidecar)
- Visual Studio 2022 Build Tools (C++ workload)
- Node.js 18+
- Tauri CLI v2 (`cargo install tauri-cli@^2`)
- Windows 10/11 (target platform)

### Build & run

```bash
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

```bash
cd src-tauri
cargo test
```

Expected test count: ~45 passing across all modules.

## Project Structure

```
src-tauri/src/
  lib.rs                # Orchestrator, 1s tick loop, system-update event
  cpu_monitor.rs        # Per-core CPU usage + frequency (sysinfo)
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
  lhm_bridge.rs         # Sidecar spawn + JSON parse
  appbar.rs             # Windows SHAppBarMessage register/unregister
  settings.rs           # Persistent settings + v1 migration
  tray.rs               # System tray icon & menu
sidecar/
  Program.cs            # LibreHardwareMonitorLib wrapper
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
docs/superpowers/
  specs/2026-05-17-sysmonitor-widget-design.md
  plans/2026-05-17-sysmonitor-implementation-plan.md
```

## License

MIT (this project). LibreHardwareMonitorLib is MPL-2.0 — bundled DLL ships with its license notice in the installer.
