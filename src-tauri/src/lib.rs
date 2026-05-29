pub mod appbar;
pub mod compact_menu;
pub mod corner_position;
pub mod cpu_monitor;
pub mod cpu_temp;
pub mod disk_monitor;
pub mod event_logger;
pub mod gpu_monitor;
pub mod lhm_bridge;
pub mod memory_monitor;
pub mod network_info;
pub mod network_monitor;
pub mod ping_monitor;
pub mod process_monitor;
pub mod session_tracker;
pub mod settings;
pub mod system_info;
pub mod tray;

use std::sync::{Arc, Mutex, OnceLock, RwLock};
use tauri::{AppHandle, Emitter, Listener, Manager};

pub struct AppState {
    pub net_monitor: Mutex<network_monitor::NetworkMonitor>,
    pub ping_monitor: Mutex<ping_monitor::PingMonitor>,
    pub net_info: Mutex<network_info::NetworkInfoCollector>,
    pub session: Mutex<session_tracker::SessionTracker>,
    pub event_log: Mutex<event_logger::EventLogger>,
    pub cached_info: Mutex<network_info::NetworkInfo>,
    pub cached_ping: Mutex<ping_monitor::PingResult>,
    pub cached_connections: Mutex<u32>,

    pub cpu: Mutex<cpu_monitor::CpuMonitor>,
    pub mem: Mutex<memory_monitor::MemoryMonitor>,
    pub disk: Mutex<disk_monitor::DiskMonitor>,
    pub proc: Mutex<process_monitor::ProcessMonitor>,
    pub lhm: lhm_bridge::LhmBridge,
    pub sys_info: system_info::SystemInfoCache,

    /// In-memory cache of `display_mode` so the visibility sentinel doesn't
    /// have to re-read + parse settings.json from disk on every tick (was
    /// 10 file reads/sec — measurable CPU + disk burn). Single writer is
    /// set_display_mode/set_compact_position; many readers (sentinel).
    pub display_mode: Arc<RwLock<String>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemUpdate {
    pub speed: network_monitor::SpeedData,
    pub ping: ping_monitor::PingResult,
    pub net_info: network_info::NetworkInfo,
    pub session: session_tracker::SessionStats,
    pub events: Vec<event_logger::NetworkEvent>,
    pub cpu: cpu_monitor::CpuStats,
    pub mem: memory_monitor::MemStats,
    pub disk: disk_monitor::DiskStats,
    pub proc: process_monitor::ProcStats,
    pub gpu: gpu_monitor::GpuStats,
}

// Tracks the currently-registered AppBar window (if any) so we can unregister
// on panic or process exit to avoid leaving permanently-reserved screen space.
static APPBAR_HWND: OnceLock<std::sync::Mutex<Option<isize>>> = OnceLock::new();
fn appbar_slot() -> &'static std::sync::Mutex<Option<isize>> {
    APPBAR_HWND.get_or_init(|| std::sync::Mutex::new(None))
}

#[tauri::command]
fn get_settings() -> settings::AppSettings {
    settings::load_settings()
}

#[tauri::command]
fn set_settings(new_settings: settings::AppSettings) -> Result<(), String> {
    settings::save_settings(&new_settings)
}

#[tauri::command]
async fn get_system_info() -> Result<system_info::SystemInfo, String> {
    // Run on a dedicated blocking thread so WMI's COM init succeeds.
    // The main app thread / Tauri command threads have COM apartment state
    // (often STA) that conflicts with wmi's MTA requirement.
    tauri::async_runtime::spawn_blocking(system_info::collect)
        .await
        .map_err(|e| e.to_string())
}

/// Move the compact window to a named corner preset and persist the choice.
/// Also flips display mode to compact_float so the bar floats wherever you want.
#[tauri::command]
fn set_compact_position(app: AppHandle, corner: String) -> Result<(), String> {
    use tauri::{LogicalPosition, Manager, Position};

    let parsed = corner_position::Corner::from_str(&corner)
        .ok_or_else(|| format!("unknown corner: {corner}"))?;

    let compact = app
        .get_webview_window("compact")
        .ok_or_else(|| "compact window not found".to_string())?;

    // If we're docked as AppBar, unregister first so we can move freely.
    if let Some(hwnd) = appbar_slot().lock().unwrap().take() {
        appbar::unregister(hwnd);
    }

    let monitor = compact
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no current monitor".to_string())?;
    let scale = monitor.scale_factor();
    let mw = (monitor.size().width as f64 / scale) as i32;
    let mh = (monitor.size().height as f64 / scale) as i32;
    let ws = compact.outer_size().map_err(|e| e.to_string())?;
    let ww = (ws.width as f64 / scale) as i32;
    let wh = (ws.height as f64 / scale) as i32;

    let taskbar_offset = estimate_taskbar_offset(parsed);
    let (x, y) = corner_position::compute_corner_position(mw, mh, ww, wh, parsed, taskbar_offset);

    compact
        .set_position(Position::Logical(LogicalPosition::new(x as f64, y as f64)))
        .map_err(|e| e.to_string())?;
    compact.show().ok();

    // Switch UI semantics to floating so the bar stays where placed
    if let Some(m) = app.get_webview_window("main") {
        m.hide().ok();
    }

    // Persist
    let mut s = settings::load_settings();
    s.compact_position = parsed.as_str().to_string();
    s.compact_x = x as f64;
    s.compact_y = y as f64;
    s.display_mode = "compact_float".to_string();
    settings::save_settings(&s)?;

    // Also update the in-memory cache the sentinel reads
    let st = app.state::<AppState>();
    *st.display_mode.write().unwrap() = "compact_float".to_string();
    Ok(())
}

/// Best-effort guess at Windows taskbar height in logical pixels.
/// Only used for bottom corners; top corners pass 0.
fn estimate_taskbar_offset(corner: corner_position::Corner) -> i32 {
    use corner_position::Corner;
    match corner {
        Corner::BottomLeft | Corner::BottomRight => 48,
        Corner::TopLeft | Corner::TopRight => 0,
    }
}

/// Persist current compact window position whenever the user drags it.
#[tauri::command]
fn save_compact_position(app: AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let compact = app
        .get_webview_window("compact")
        .ok_or_else(|| "compact window not found".to_string())?;
    let pos = compact.outer_position().map_err(|e| e.to_string())?;
    let monitor = compact
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no current monitor".to_string())?;
    let scale = monitor.scale_factor();
    let lx = pos.x as f64 / scale;
    let ly = pos.y as f64 / scale;

    let mut s = settings::load_settings();
    s.compact_x = lx;
    s.compact_y = ly;
    s.compact_position = "custom".to_string();
    settings::save_settings(&s)
}

/// Build the full (7-tab) main window on demand. Previously this was
/// declared in tauri.conf.json `windows` and got created at app startup
/// even though it stayed hidden — that booted ~6 WebView2 helper processes
/// and ~250MB of RAM for the 95% of users who only use the compact bar.
///
/// Same dimensions and chrome flags as the old static declaration.
fn build_main_window(app: &AppHandle) -> tauri::Result<tauri::WebviewWindow> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};
    WebviewWindowBuilder::new(app, "main", WebviewUrl::App("/".into()))
        .title("SysMonitor")
        .inner_size(420.0, 440.0)
        .decorations(false)
        .always_on_top(true)
        .transparent(false)
        .resizable(false)
        .skip_taskbar(true)
        .center()
        .visible(false)
        .build()
}

/// Show a NATIVE OS-level context menu on right-click of the compact bar.
/// The previous in-page Svelte menu got clipped because the compact window
/// is only ~28px tall; a native menu is rendered by the OS and isn't
/// constrained by the window's client area at all.
///
/// Event handling is global (see `.on_menu_event` in run()) — the same
/// handler that drives tray menu items also drives these popup items,
/// dispatched by ID.
#[tauri::command]
fn show_compact_menu(app: AppHandle) -> Result<(), String> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
    use tauri::Manager;

    let webview = app
        .get_webview_window("compact")
        .ok_or_else(|| "compact window not found".to_string())?;

    let layout = compact_menu::compact_menu_layout();
    let mut builder = MenuBuilder::new(&app);
    for entry in &layout {
        builder = match entry {
            compact_menu::MenuEntry::Item(id, label) => builder.item(
                &MenuItemBuilder::with_id(*id, *label)
                    .build(&app)
                    .map_err(|e| e.to_string())?,
            ),
            compact_menu::MenuEntry::Separator => builder.item(
                &PredefinedMenuItem::separator(&app).map_err(|e| e.to_string())?,
            ),
        };
    }
    let menu = builder.build().map_err(|e| e.to_string())?;

    webview.popup_menu(&menu).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_display_mode(app: AppHandle, mode: String) -> Result<(), String> {
    // Update the in-memory cache FIRST so the visibility sentinel reads
    // the new mode on its next tick (within ~500ms), before any window
    // operations below.
    {
        let st = app.state::<AppState>();
        *st.display_mode.write().unwrap() = mode.clone();
    }

    // Always unregister any existing AppBar first
    if let Some(hwnd) = appbar_slot().lock().unwrap().take() {
        appbar::unregister(hwnd);
    }

    let compact = app.get_webview_window("compact");

    match mode.as_str() {
        "full" => {
            if let Some(c) = &compact {
                c.hide().ok();
            }
            // Lazy-create the main (full) window on first show. Keeping it
            // out of tauri.conf.json saves ~6 WebView2 helper processes and
            // ~250MB RAM for the 95% of users who only ever use the bar.
            let main = match app.get_webview_window("main") {
                Some(w) => w,
                None => match build_main_window(&app) {
                    Ok(w) => w,
                    Err(e) => return Err(format!("create main window: {e}")),
                },
            };
            main.show().ok();
            main.set_focus().ok();
        }
        "compact_appbar" => {
            if let Some(m) = app.get_webview_window("main") {
                m.hide().ok();
            }
            if let Some(c) = &compact {
                c.show().ok();
                #[cfg(target_os = "windows")]
                {
                    if let Ok(hwnd) = c.hwnd() {
                        let h_isize = hwnd.0 as isize;
                        let edge_str = settings::load_settings().appbar_edge;
                        let edge = appbar::Edge::from_str(&edge_str);
                        appbar::register(h_isize, edge, 30);
                        *appbar_slot().lock().unwrap() = Some(h_isize);
                    }
                }
            }
        }
        "compact_float" => {
            if let Some(m) = app.get_webview_window("main") {
                m.hide().ok();
            }
            if let Some(c) = &compact {
                use tauri::{LogicalPosition, Position};
                let s = settings::load_settings();
                let restored = (s.compact_x, s.compact_y);
                let on_screen = if let Ok(Some(monitor)) = c.current_monitor() {
                    let scale = monitor.scale_factor();
                    let mw = (monitor.size().width as f64 / scale) as i32;
                    let mh = (monitor.size().height as f64 / scale) as i32;
                    let ws = c.outer_size().ok();
                    let (ww, wh) = ws
                        .map(|s| (
                            (s.width as f64 / scale) as i32,
                            (s.height as f64 / scale) as i32,
                        ))
                        .unwrap_or((360, 30));
                    corner_position::position_is_on_screen(
                        restored.0, restored.1, ww, wh, mw, mh, 40,
                    )
                } else {
                    false
                };

                if (restored.0 != 0.0 || restored.1 != 0.0) && on_screen {
                    c.set_position(Position::Logical(LogicalPosition::new(restored.0, restored.1)))
                        .ok();
                } else {
                    // Fall back to the saved corner preset (default bottom-left)
                    let corner = corner_position::Corner::from_str(&s.compact_position)
                        .unwrap_or(corner_position::Corner::BottomLeft);
                    if let Ok(Some(monitor)) = c.current_monitor() {
                        let scale = monitor.scale_factor();
                        let mw = (monitor.size().width as f64 / scale) as i32;
                        let mh = (monitor.size().height as f64 / scale) as i32;
                        let ws = c.outer_size().ok();
                        let (ww, wh) = ws
                            .map(|s| (
                                (s.width as f64 / scale) as i32,
                                (s.height as f64 / scale) as i32,
                            ))
                            .unwrap_or((360, 30));
                        let tb = match corner {
                            corner_position::Corner::BottomLeft
                            | corner_position::Corner::BottomRight => 48,
                            _ => 0,
                        };
                        let (x, y) = corner_position::compute_corner_position(
                            mw, mh, ww, wh, corner, tb,
                        );
                        c.set_position(Position::Logical(LogicalPosition::new(x as f64, y as f64)))
                            .ok();
                    }
                }
                c.show().ok();
                // Re-assert always-on-top — Windows may demote frameless
                // windows when other topmost windows (taskbar/start menu) gain focus
                c.set_always_on_top(true).ok();
                c.set_focus().ok();
            }
        }
        "tray_only" => {
            if let Some(m) = app.get_webview_window("main") {
                m.hide().ok();
            }
            if let Some(c) = &compact {
                c.hide().ok();
            }
        }
        _ => return Err(format!("unknown mode: {mode}")),
    }

    // Persist
    let mut s = settings::load_settings();
    s.display_mode = mode;
    settings::save_settings(&s)?;
    Ok(())
}

fn start_monitoring(app: AppHandle) {
    std::thread::spawn(move || {
        let mut tick: u32 = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            tick += 1;
            let state = app.state::<AppState>();

            let speed = state.net_monitor.lock().unwrap().poll();

            if tick % 2 == 0 {
                let ping = state.ping_monitor.lock().unwrap().ping();
                *state.cached_ping.lock().unwrap() = ping;
            }
            let ping = state.cached_ping.lock().unwrap().clone();

            // PERF: was every 5s — `netsh wlan show interfaces` +
            // `ipconfig /all` are subprocess spawns (~50-100ms each, run
            // synchronously). Local IP, SSID, gateway, DNS, MAC almost
            // never change inside a session, so 30s is plenty. Public IP
            // is cached separately for 5min inside the collector.
            if tick % 30 == 0 || tick == 1 {
                let info = state.net_info.lock().unwrap().collect();
                *state.cached_info.lock().unwrap() = info;
            }
            let net_info = state.cached_info.lock().unwrap().clone();

            // PERF: was every 5s — netstat enumerates EVERY TCP connection.
            // 15s gives a near-live count without burning CPU.
            if tick % 15 == 0 || tick == 1 {
                let conn = settings::get_active_connections();
                *state.cached_connections.lock().unwrap() = conn;
            }
            let conn_count = *state.cached_connections.lock().unwrap();

            state
                .session
                .lock()
                .unwrap()
                .update(speed.download_bps, speed.upload_bps);
            let session_stats = state.session.lock().unwrap().get_stats(conn_count);

            state.event_log.lock().unwrap().check_events(
                speed.download_bps,
                &net_info.local_ip,
                &net_info.connection_type,
            );
            let events = state.event_log.lock().unwrap().get_events();

            let mut cpu = state.cpu.lock().unwrap().poll();
            let mem = state.mem.lock().unwrap().poll();
            let disk = state.disk.lock().unwrap().poll();
            let proc = state.proc.lock().unwrap().poll(10);

            let available = state
                .lhm
                .available
                .load(std::sync::atomic::Ordering::Relaxed);
            let gpu = gpu_monitor::read(&state.lhm.reading, available);

            // CPU temperature: prefer sidecar reading, fall back to WMI thermal zone
            cpu.temp_c = state
                .lhm
                .reading
                .lock()
                .unwrap()
                .as_ref()
                .and_then(|r| r.cpu.temp_c)
                .or_else(cpu_temp::read_cpu_temp_c);

            let update = SystemUpdate {
                speed,
                ping,
                net_info,
                session: session_stats,
                events,
                cpu,
                mem,
                disk,
                proc,
                gpu,
            };

            app.emit("system-update", &update).ok();
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Cleanup AppBar registration on panic — otherwise the OS holds the
    // reserved screen-edge slot until next reboot.
    std::panic::set_hook(Box::new(|p| {
        if let Some(hwnd) = appbar_slot().lock().ok().and_then(|mut g| g.take()) {
            appbar::unregister(hwnd);
        }
        eprintln!("panic: {p}");
    }));

    settings::migrate_v1_settings_if_present();

    tauri::Builder::default()
        // CRITICAL: single-instance lock MUST be the first plugin registered.
        // Without it, a second launch (e.g. user clicks "Run as administrator"
        // while a non-elevated instance is already running) tries to spin up
        // its own WebView2 against the same %LOCALAPPDATA%/com.sysmonitor.app
        // /EBWebView user-data folder. WebView2 cannot share that folder
        // across processes — the second launch fails silently and exits.
        // User sees "Run as admin did nothing" because the bar was already
        // visible from the existing instance.
        //
        // With this plugin, the second launch detects the lock, fires this
        // callback in the *existing* process, and exits cleanly. We bring the
        // existing bar to the front so the user gets visible feedback.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            use tauri::Manager;
            if let Some(c) = app.get_webview_window("compact") {
                let _ = c.show();
                let _ = c.set_always_on_top(true);
                let _ = c.set_focus();
            }
            if let Some(m) = app.get_webview_window("main") {
                if m.is_visible().unwrap_or(false) {
                    let _ = m.set_focus();
                }
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        // App-wide menu event handler — fires for both tray menu and any
        // popup menus (compact bar's right-click). Dispatches by ID to the
        // same event names the tray menu already uses.
        .on_menu_event(|app, event| {
            use tauri::Manager;
            match event.id.as_ref() {
                "cpos_tl" => { app.emit("set-compact-position", "top-left").ok(); }
                "cpos_tr" => { app.emit("set-compact-position", "top-right").ok(); }
                "cpos_bl" => { app.emit("set-compact-position", "bottom-left").ok(); }
                "cpos_br" => { app.emit("set-compact-position", "bottom-right").ok(); }
                "mode_full" => { app.emit("set-mode", "full").ok(); }
                "mode_appbar" => { app.emit("set-mode", "compact_appbar").ok(); }
                "mode_float" => { app.emit("set-mode", "compact_float").ok(); }
                "mode_tray" => { app.emit("set-mode", "tray_only").ok(); }
                "retry_sensors" => { app.emit("retry-sensors", ()).ok(); }
                "restart_admin" => { app.emit("restart-as-admin", ()).ok(); }
                "always_on_top" => {
                    if let Some(c) = app.get_webview_window("compact") {
                        if let Ok(cur) = c.is_always_on_top() {
                            let _ = c.set_always_on_top(!cur);
                        }
                    }
                }
                "quit" => { app.exit(0); }
                _ => {}
            }
        })
        .manage(AppState {
            net_monitor: Mutex::new(network_monitor::NetworkMonitor::new()),
            ping_monitor: Mutex::new(ping_monitor::PingMonitor::new()),
            net_info: Mutex::new(network_info::NetworkInfoCollector::new()),
            session: Mutex::new(session_tracker::SessionTracker::new()),
            event_log: Mutex::new(event_logger::EventLogger::new()),
            cached_info: Mutex::new(network_info::NetworkInfo::default()),
            cached_ping: Mutex::new(ping_monitor::PingResult {
                latency_ms: None,
                status: ping_monitor::PingStatus::Ok,
            }),
            cached_connections: Mutex::new(0),
            cpu: Mutex::new(cpu_monitor::CpuMonitor::new()),
            mem: Mutex::new(memory_monitor::MemoryMonitor::new()),
            disk: Mutex::new(disk_monitor::DiskMonitor::new()),
            proc: Mutex::new(process_monitor::ProcessMonitor::new()),
            lhm: lhm_bridge::LhmBridge::new(),
            sys_info: system_info::SystemInfoCache::default(),
            display_mode: Arc::new(RwLock::new(
                settings::load_settings().display_mode,
            )),
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_system_info,
            lhm_bridge::retry_sensors,
            set_display_mode,
            set_compact_position,
            save_compact_position,
            show_compact_menu
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            tray::create_tray(&handle).expect("failed to create tray");

            // Spawn LHM sidecar bridge
            let state = handle.state::<AppState>();
            lhm_bridge::start(&handle, &state.lhm);

            // Listen for tray events that change display mode or retry sensors
            let h_mode = handle.clone();
            handle.listen("set-mode", move |event| {
                let mode = event.payload().trim_matches('"').to_string();
                let _ = set_display_mode(h_mode.clone(), mode);
            });
            let h_retry = handle.clone();
            handle.listen("retry-sensors", move |_| {
                let state = h_retry.state::<AppState>();
                state
                    .lhm
                    .restart_count
                    .store(0, std::sync::atomic::Ordering::Relaxed);
                state
                    .lhm
                    .available
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            });

            // Listen for tray "Compact Position" submenu selections
            let h_pos = handle.clone();
            handle.listen("set-compact-position", move |event| {
                let corner = event.payload().trim_matches('"').to_string();
                let _ = set_compact_position(h_pos.clone(), corner);
            });

            // Listen for "Restart as administrator" — Win32 ShellExecuteW
            // with "runas" verb spawns an elevated copy, then we exit so
            // the single-instance lock releases and the elevated instance
            // takes over. If user declines UAC, we stay running.
            let h_admin = handle.clone();
            handle.listen("restart-as-admin", move |_| {
                let exe = match std::env::current_exe() {
                    Ok(p) => p,
                    Err(e) => { eprintln!("current_exe failed: {e}"); return; }
                };
                match appbar::restart_as_admin(&exe) {
                    Ok(()) => {
                        // Give the new process a beat to grab the single-
                        // instance lock before we exit ours.
                        let h2 = h_admin.clone();
                        tauri::async_runtime::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            h2.exit(0);
                        });
                    }
                    Err(e) => {
                        eprintln!("restart_as_admin failed: {e}");
                        // User likely declined UAC — keep running.
                    }
                }
            });

            // Start the per-second monitoring loop
            start_monitoring(handle.clone());

            // ===== COMPACT BAR VISIBILITY SENTINEL =====
            // Two layers of defense against the bar disappearing:
            //
            // 1) WS_EX_NOACTIVATE applied once at startup (below). Prevents
            //    the window from EVER becoming foreground, so Windows'
            //    tool-window auto-hide on shell activation doesn't fire.
            //
            // 2) Polled sentinel. Belt-and-suspenders — restores visibility
            //    if anything slips through the NOACTIVATE defense.
            //
            // PERF: was 100ms polling + settings::load_settings() (file I/O)
            //       per tick = 10 disk reads/sec + 10 JSON parses/sec.
            //       Now 500ms polling reading from an in-memory RwLock that
            //       set_display_mode/set_compact_position update. 500ms is
            //       still visually instant for "the bar came back" and is
            //       20x cheaper than the old 100ms-with-disk-read.
            const MIN_W: u32 = 200;
            const MIN_H: u32 = 24;
            let h_pin = handle.clone();
            let mode_cache = handle.state::<AppState>().display_mode.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    // Read from in-memory cache, NOT from settings.json
                    let should_be_visible = {
                        let mode = mode_cache.read().unwrap();
                        mode.as_str() == "compact_float" || mode.as_str() == "compact_appbar"
                    };
                    if !should_be_visible {
                        continue;
                    }
                    let Some(c) = h_pin.get_webview_window("compact") else { continue };

                    let visible = c.is_visible().unwrap_or(true);
                    let minimized = c.is_minimized().unwrap_or(false);

                    if !visible {
                        let _ = c.show();
                    }
                    if minimized {
                        let _ = c.unminimize();
                    }
                    // Only call set_always_on_top if it's NOT already topmost.
                    // Each call hits Win32 SetWindowPos which generates
                    // WM_WINDOWPOSCHANGED — wasteful at 10Hz unconditionally.
                    if !c.is_always_on_top().unwrap_or(true) {
                        let _ = c.set_always_on_top(true);
                    }

                    if let Ok(size) = c.outer_size() {
                        // Snap back if too small (bug guard)
                        if size.width < MIN_W || size.height < MIN_H {
                            let _ = c.set_size(tauri::LogicalSize::new(620u32, 28u32));
                            let _ = c.show();
                        }
                        // Snap back if too tall (window grew beyond bar height)
                        const MAX_H_PHYS: u32 = 120;
                        if size.height > MAX_H_PHYS {
                            let w_logical = (size.width as f64 / 1.0).max(620.0) as u32;
                            let _ = c.set_size(tauri::LogicalSize::new(w_logical, 28u32));
                        }
                    }
                }
            });

            // Apply WS_EX_NOACTIVATE to the compact window once at startup.
            // Has to wait a beat for the window to fully exist + be styled
            // by Tauri before our flags stick.
            #[cfg(target_os = "windows")]
            {
                let h_style = handle.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    if let Some(c) = h_style.get_webview_window("compact") {
                        if let Ok(hwnd) = c.hwnd() {
                            appbar::apply_widget_styles(hwnd.0 as isize);
                        }
                    }
                });
            }

            // Apply the saved display mode (defaults to compact_appbar on first run)
            let h_init = handle.clone();
            let saved_mode = settings::load_settings().display_mode;
            tauri::async_runtime::spawn(async move {
                let _ = set_display_mode(h_init, saved_mode);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
