use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub window_x: f64,
    pub window_y: f64,
    pub position_preset: String,
    pub always_on_top: bool,
    pub start_on_boot: bool,
    pub selected_tab: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            window_x: 0.0,
            window_y: 0.0,
            position_preset: "top-right".to_string(),
            always_on_top: true,
            start_on_boot: false,
            selected_tab: "speed".to_string(),
        }
    }
}

fn settings_path() -> PathBuf {
    let app_data = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = app_data.join("sysmonitor");
    fs::create_dir_all(&dir).ok();
    dir.join("settings.json")
}

pub fn load_settings() -> AppSettings {
    let path = settings_path();
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path();
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

/// Count ESTABLISHED TCP connections on non-loopback interfaces
pub fn get_active_connections() -> u32 {
    let mut cmd = std::process::Command::new("netstat");
    cmd.args(["-n", "-p", "TCP"]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = cmd.output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .filter(|line| {
                    line.contains("ESTABLISHED")
                        && !line.contains("127.0.0.1")
                        && !line.contains("[::1]")
                })
                .count() as u32
        }
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.position_preset, "top-right");
        assert!(settings.always_on_top);
        assert!(!settings.start_on_boot);
        assert_eq!(settings.selected_tab, "speed");
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.position_preset, settings.position_preset);
        assert_eq!(parsed.always_on_top, settings.always_on_top);
    }

    #[test]
    fn test_settings_folder_is_sysmonitor() {
        let path = settings_path();
        let parent = path.parent().expect("settings path must have parent");
        let folder_name = parent.file_name().unwrap().to_string_lossy();
        assert_eq!(folder_name, "sysmonitor");
    }
}
