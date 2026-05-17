use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub cpu_pct: u8,
    pub cpu_temp_c: u8,
    pub gpu_temp_c: u8,
    pub ram_pct: u8,
    pub disk_free_pct: u8,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            cpu_pct: 80,
            cpu_temp_c: 80,
            gpu_temp_c: 80,
            ram_pct: 85,
            disk_free_pct: 15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub window_x: f64,
    pub window_y: f64,
    pub position_preset: String,
    pub always_on_top: bool,
    pub start_on_boot: bool,
    pub selected_tab: String,

    #[serde(default = "default_display_mode")]
    pub display_mode: String,
    #[serde(default = "default_appbar_edge")]
    pub appbar_edge: String,
    #[serde(default = "default_temp_unit")]
    pub temp_unit: String,
    #[serde(default)]
    pub warning_thresholds: Thresholds,
    #[serde(default = "default_true")]
    pub sidecar_enabled: bool,

    #[serde(default = "default_compact_position")]
    pub compact_position: String, // last preset chosen: "top-left" | "top-right" | "bottom-left" | "bottom-right" | "custom"
    #[serde(default)]
    pub compact_x: f64, // 0.0 means unset; populated after first move
    #[serde(default)]
    pub compact_y: f64,
}

fn default_display_mode() -> String { "compact_float".to_string() }
fn default_appbar_edge() -> String { "top".to_string() }
fn default_temp_unit() -> String { "c".to_string() }
fn default_true() -> bool { true }
fn default_compact_position() -> String { "bottom-left".to_string() }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            window_x: 0.0,
            window_y: 0.0,
            position_preset: "top-right".to_string(),
            always_on_top: true,
            start_on_boot: false,
            selected_tab: "overview".to_string(),
            display_mode: default_display_mode(),
            appbar_edge: default_appbar_edge(),
            temp_unit: default_temp_unit(),
            warning_thresholds: Thresholds::default(),
            sidecar_enabled: true,
            compact_position: default_compact_position(),
            compact_x: 0.0,
            compact_y: 0.0,
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
        assert_eq!(settings.selected_tab, "overview");
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

    #[test]
    fn default_display_mode_is_compact_float() {
        let s = AppSettings::default();
        assert_eq!(s.display_mode, "compact_float");
    }

    #[test]
    fn default_warning_thresholds_present() {
        let s = AppSettings::default();
        assert_eq!(s.warning_thresholds.cpu_pct, 80);
        assert_eq!(s.warning_thresholds.cpu_temp_c, 80);
        assert_eq!(s.warning_thresholds.gpu_temp_c, 80);
        assert_eq!(s.warning_thresholds.ram_pct, 85);
        assert_eq!(s.warning_thresholds.disk_free_pct, 15);
    }

    #[test]
    fn deserializing_v1_json_uses_defaults_for_new_fields() {
        let v1_json = r#"{
          "window_x":0.0,"window_y":0.0,
          "position_preset":"top-right","always_on_top":true,
          "start_on_boot":false,"selected_tab":"speed"
        }"#;
        let s: AppSettings = serde_json::from_str(v1_json).expect("must deserialize");
        assert_eq!(s.display_mode, "compact_float");
        assert_eq!(s.appbar_edge, "top");
        assert_eq!(s.temp_unit, "c");
        assert_eq!(s.warning_thresholds.cpu_pct, 80);
        assert!(s.sidecar_enabled);
    }

    #[test]
    fn default_compact_position_is_bottom_left() {
        let s = AppSettings::default();
        assert_eq!(s.compact_position, "bottom-left");
        assert_eq!(s.compact_x, 0.0);
        assert_eq!(s.compact_y, 0.0);
    }

    #[test]
    fn deserializing_without_compact_fields_uses_defaults() {
        // Pre-existing v2 settings file from before this feature was added —
        // missing compact_position, compact_x, compact_y must use serde defaults
        let json = r#"{
          "window_x":0.0,"window_y":0.0,
          "position_preset":"top-right","always_on_top":true,
          "start_on_boot":false,"selected_tab":"overview",
          "display_mode":"compact_float","appbar_edge":"top","temp_unit":"c",
          "warning_thresholds":{"cpu_pct":80,"cpu_temp_c":80,"gpu_temp_c":80,"ram_pct":85,"disk_free_pct":15},
          "sidecar_enabled":true
        }"#;
        let s: AppSettings = serde_json::from_str(json).expect("must deserialize");
        assert_eq!(s.compact_position, "bottom-left");
        assert_eq!(s.compact_x, 0.0);
        assert_eq!(s.compact_y, 0.0);
    }

    #[test]
    fn migrate_from_v1_path_returns_copy() {
        use std::fs;
        let tmp = std::env::temp_dir().join("sysmon_test_v1");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let v1_dir = tmp.join("netmonitor");
        fs::create_dir_all(&v1_dir).unwrap();
        fs::write(
            v1_dir.join("settings.json"),
            r#"{"window_x":1.0,"window_y":2.0,"position_preset":"top-left","always_on_top":false,"start_on_boot":true,"selected_tab":"network"}"#,
        )
        .unwrap();

        let s = try_migrate_from(&v1_dir).expect("should migrate");
        assert_eq!(s.window_x, 1.0);
        assert_eq!(s.position_preset, "top-left");
        assert!(!s.always_on_top);
        assert!(s.start_on_boot);
    }
}

pub fn try_migrate_from(v1_dir: &std::path::Path) -> Option<AppSettings> {
    let path = v1_dir.join("settings.json");
    if !path.exists() {
        return None;
    }
    let json = std::fs::read_to_string(&path).ok()?;
    let parsed: AppSettings = serde_json::from_str(&json).ok()?;
    Some(parsed)
}

pub fn migrate_v1_settings_if_present() {
    let app_data = match dirs::config_dir() {
        Some(d) => d,
        None => return,
    };
    let v1 = app_data.join("netmonitor");
    let v2 = app_data.join("sysmonitor");
    if v2.join("settings.json").exists() {
        return;
    }
    if let Some(s) = try_migrate_from(&v1) {
        let _ = std::fs::create_dir_all(&v2);
        if let Ok(json) = serde_json::to_string_pretty(&s) {
            let _ = std::fs::write(v2.join("settings.json"), json);
        }
    }
}
