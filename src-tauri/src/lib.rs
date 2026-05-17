pub mod appbar;
pub mod cpu_monitor;
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

use std::sync::{Mutex, OnceLock};
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
fn get_system_info(state: tauri::State<AppState>) -> system_info::SystemInfo {
    state.sys_info.get_or_collect()
}

#[tauri::command]
fn set_display_mode(app: AppHandle, mode: String) -> Result<(), String> {
    // Always unregister any existing AppBar first
    if let Some(hwnd) = appbar_slot().lock().unwrap().take() {
        appbar::unregister(hwnd);
    }

    let main = app.get_webview_window("main");
    let compact = app.get_webview_window("compact");

    match mode.as_str() {
        "full" => {
            if let Some(c) = &compact {
                c.hide().ok();
            }
            if let Some(m) = &main {
                m.show().ok();
                m.set_focus().ok();
            }
        }
        "compact_appbar" => {
            if let Some(m) = &main {
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
            if let Some(m) = &main {
                m.hide().ok();
            }
            if let Some(c) = &compact {
                c.show().ok();
            }
        }
        "tray_only" => {
            if let Some(m) = &main {
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

            if tick % 5 == 0 || tick == 1 {
                let info = state.net_info.lock().unwrap().collect();
                *state.cached_info.lock().unwrap() = info;
            }
            let net_info = state.cached_info.lock().unwrap().clone();

            if tick % 5 == 0 || tick == 1 {
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

            let cpu = state.cpu.lock().unwrap().poll();
            let mem = state.mem.lock().unwrap().poll();
            let disk = state.disk.lock().unwrap().poll();
            let proc = state.proc.lock().unwrap().poll(10);

            let available = state
                .lhm
                .available
                .load(std::sync::atomic::Ordering::Relaxed);
            let gpu = gpu_monitor::read(&state.lhm.reading, available);

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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
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
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_system_info,
            lhm_bridge::retry_sensors,
            set_display_mode
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

            // Start the per-second monitoring loop
            start_monitoring(handle.clone());

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
