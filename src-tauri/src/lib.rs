pub mod event_logger;
pub mod network_info;
pub mod network_monitor;
pub mod ping_monitor;
pub mod session_tracker;
pub mod settings;
pub mod tray;

use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub struct AppState {
    pub net_monitor: Mutex<network_monitor::NetworkMonitor>,
    pub ping_monitor: Mutex<ping_monitor::PingMonitor>,
    pub net_info: Mutex<network_info::NetworkInfoCollector>,
    pub session: Mutex<session_tracker::SessionTracker>,
    pub event_log: Mutex<event_logger::EventLogger>,
    pub cached_info: Mutex<network_info::NetworkInfo>,
    pub cached_ping: Mutex<ping_monitor::PingResult>,
    pub cached_connections: Mutex<u32>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NetworkUpdate {
    pub speed: network_monitor::SpeedData,
    pub ping: ping_monitor::PingResult,
    pub info: network_info::NetworkInfo,
    pub session: session_tracker::SessionStats,
    pub events: Vec<event_logger::NetworkEvent>,
}

#[tauri::command]
fn get_settings() -> settings::AppSettings {
    settings::load_settings()
}

#[tauri::command]
fn set_settings(new_settings: settings::AppSettings) -> Result<(), String> {
    settings::save_settings(&new_settings)
}

fn start_monitoring(app: AppHandle) {
    std::thread::spawn(move || {
        let mut tick: u32 = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            tick += 1;

            let state = app.state::<AppState>();

            // Speed (every tick)
            let speed = state.net_monitor.lock().unwrap().poll();

            // Ping (every 2 ticks)
            if tick % 2 == 0 {
                let ping = state.ping_monitor.lock().unwrap().ping();
                *state.cached_ping.lock().unwrap() = ping;
            }
            let ping = state.cached_ping.lock().unwrap().clone();

            // Network info (every 5 ticks or first tick)
            if tick % 5 == 0 || tick == 1 {
                let info = state.net_info.lock().unwrap().collect();
                *state.cached_info.lock().unwrap() = info;
            }
            let info = state.cached_info.lock().unwrap().clone();

            // Active connections (every 5 ticks or first tick)
            if tick % 5 == 0 || tick == 1 {
                let conn = settings::get_active_connections();
                *state.cached_connections.lock().unwrap() = conn;
            }
            let conn_count = *state.cached_connections.lock().unwrap();

            // Session update
            state
                .session
                .lock()
                .unwrap()
                .update(speed.download_bps, speed.upload_bps);
            let session_stats = state.session.lock().unwrap().get_stats(conn_count);

            // Event detection
            state.event_log.lock().unwrap().check_events(
                speed.download_bps,
                &info.local_ip,
                &info.connection_type,
            );
            let events = state.event_log.lock().unwrap().get_events();

            let update = NetworkUpdate {
                speed,
                ping,
                info,
                session: session_stats,
                events,
            };

            app.emit("network-update", &update).ok();
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
        })
        .invoke_handler(tauri::generate_handler![get_settings, set_settings])
        .setup(|app| {
            let handle = app.handle().clone();
            tray::create_tray(&handle).expect("failed to create tray");
            start_monitoring(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
