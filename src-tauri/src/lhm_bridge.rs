use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpuReading {
    pub name: String,
    pub load_pct: Option<f32>,
    pub temp_c: Option<f32>,
    pub vram_used_mb: u32,
    pub vram_total_mb: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuReading {
    pub temp_c: Option<f32>,
    pub package_w: Option<f32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareReading {
    pub ts: u64,
    pub cpu: CpuReading,
    pub gpus: Vec<GpuReading>,
}

pub type SharedReading = Arc<Mutex<Option<HardwareReading>>>;

pub fn parse_line(line: &str) -> Option<HardwareReading> {
    serde_json::from_str(line.trim()).ok()
}

pub struct LhmBridge {
    pub reading: SharedReading,
    pub available: Arc<AtomicBool>,
    pub restart_count: Arc<AtomicU32>,
}

impl LhmBridge {
    pub fn new() -> Self {
        Self {
            reading: Arc::new(Mutex::new(None)),
            available: Arc::new(AtomicBool::new(true)),
            restart_count: Arc::new(AtomicU32::new(0)),
        }
    }
}

const MAX_RESTARTS: u32 = 3;
const RESTART_DELAY_MS: u64 = 5_000;

pub fn start(app: &AppHandle, bridge: &LhmBridge) {
    use tauri_plugin_shell::process::CommandEvent;
    use tauri_plugin_shell::ShellExt;

    let app = app.clone();
    let reading = bridge.reading.clone();
    let available = bridge.available.clone();
    let restart_count = bridge.restart_count.clone();

    std::thread::spawn(move || loop {
        if restart_count.load(Ordering::Relaxed) >= MAX_RESTARTS {
            available.store(false, Ordering::Relaxed);
            return;
        }

        let sidecar = match app.shell().sidecar("sysmon-sensor") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("sidecar creation failed: {e}");
                available.store(false, Ordering::Relaxed);
                return;
            }
        };

        let (mut rx, _child) = match sidecar.spawn() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("sidecar spawn failed: {e}");
                restart_count.fetch_add(1, Ordering::Relaxed);
                std::thread::sleep(std::time::Duration::from_millis(RESTART_DELAY_MS));
                continue;
            }
        };

        let mut got_any = false;
        let mut buf = String::new();

        while let Some(event) = futures::executor::block_on(rx.recv()) {
            match event {
                CommandEvent::Stdout(bytes) => {
                    if let Ok(s) = std::str::from_utf8(&bytes) {
                        buf.push_str(s);
                        while let Some(idx) = buf.find('\n') {
                            let line: String = buf.drain(..=idx).collect();
                            if let Some(r) = parse_line(&line) {
                                got_any = true;
                                *reading.lock().unwrap() = Some(r);
                            }
                        }
                    }
                }
                CommandEvent::Stderr(bytes) => {
                    if let Ok(s) = std::str::from_utf8(&bytes) {
                        eprintln!("sidecar stderr: {}", s.trim_end());
                    }
                }
                CommandEvent::Terminated(payload) => {
                    eprintln!("sidecar terminated (code={:?})", payload.code);
                    if got_any {
                        restart_count.store(0, Ordering::Relaxed);
                    } else {
                        restart_count.fetch_add(1, Ordering::Relaxed);
                    }
                    break;
                }
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(RESTART_DELAY_MS));
    });
}

#[tauri::command]
pub fn retry_sensors(state: tauri::State<crate::AppState>) {
    state.lhm.restart_count.store(0, Ordering::Relaxed);
    state.lhm.available.store(true, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"ts":1715961600000,"cpu":{"temp_c":58.3,"package_w":52.1},"gpus":[{"name":"AMD Radeon RX 7800 XT","load_pct":12.0,"temp_c":47.0,"vram_used_mb":1024,"vram_total_mb":16384}]}"#;

    #[test]
    fn parses_valid_sample_line() {
        let r = parse_line(SAMPLE).expect("must parse");
        assert_eq!(r.ts, 1715961600000);
        assert_eq!(r.cpu.temp_c, Some(58.3));
        assert_eq!(r.gpus.len(), 1);
        assert_eq!(r.gpus[0].name, "AMD Radeon RX 7800 XT");
        assert_eq!(r.gpus[0].vram_total_mb, 16384);
    }

    #[test]
    fn parses_with_null_sensors() {
        let line = r#"{"ts":1,"cpu":{"temp_c":null,"package_w":null},"gpus":[]}"#;
        let r = parse_line(line).expect("must parse");
        assert!(r.cpu.temp_c.is_none());
        assert!(r.gpus.is_empty());
    }

    #[test]
    fn returns_none_for_garbage() {
        assert!(parse_line("not json").is_none());
        assert!(parse_line("").is_none());
        assert!(parse_line("{partial").is_none());
    }

    #[test]
    fn handles_trailing_whitespace() {
        let line = format!("{}   \n", SAMPLE);
        assert!(parse_line(&line).is_some());
    }
}
