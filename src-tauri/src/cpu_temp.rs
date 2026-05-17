//! CPU temperature reader for Windows.
//!
//! Tries multiple WMI sources, falls back to None if none work. Provides a
//! best-effort temperature without requiring the LibreHardwareMonitor sidecar.
//!
//! Sources, in priority order:
//! 1. `MSAcpi_ThermalZoneTemperature` in `root\WMI` (ACPI thermal zone — usually present)
//! 2. `Win32_PerfFormattedData_Counters_ThermalZoneInformation` in `root\cimv2`
//!
//! Both report in tenths of a Kelvin × 10 (kelvin_decikelvins), so the same
//! parsing helper handles both.

/// Convert "tenths of a Kelvin" (e.g. 3032 = 303.2 K = 30.05 °C) to °C.
/// Returns None for values outside a sane CPU range (0..120 °C).
pub fn deci_kelvin_to_celsius(dk: u32) -> Option<f32> {
    let kelvin = dk as f32 / 10.0;
    let celsius = kelvin - 273.15;
    if (0.0..=120.0).contains(&celsius) {
        Some(celsius)
    } else {
        None
    }
}

/// Pick the highest temperature among multiple sensors (most useful for CPU —
/// the package is usually the hottest zone reported).
pub fn hottest(temps: &[f32]) -> Option<f32> {
    temps
        .iter()
        .copied()
        .filter(|t| t.is_finite())
        .fold(None::<f32>, |acc, t| Some(acc.map_or(t, |a| a.max(t))))
}

/// Read CPU temperature in °C, or None if no sensor source is available.
pub fn read_cpu_temp_c() -> Option<f32> {
    #[cfg(target_os = "windows")]
    {
        read_via_msacpi().or_else(read_via_perfcounter)
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn read_via_msacpi() -> Option<f32> {
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct Thermal {
        current_temperature: u32,
    }

    let com = COMLibrary::new().ok()?;
    let conn = WMIConnection::with_namespace_path("root\\WMI", com).ok()?;
    let rows: Vec<Thermal> = conn
        .raw_query("SELECT CurrentTemperature FROM MSAcpi_ThermalZoneTemperature")
        .ok()?;
    let celsius: Vec<f32> = rows
        .into_iter()
        .filter_map(|r| deci_kelvin_to_celsius(r.current_temperature))
        .collect();
    hottest(&celsius)
}

#[cfg(target_os = "windows")]
fn read_via_perfcounter() -> Option<f32> {
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct Pf {
        temperature: u32,
    }

    let com = COMLibrary::new().ok()?;
    let conn = WMIConnection::new(com).ok()?;
    let rows: Vec<Pf> = conn
        .raw_query("SELECT Temperature FROM Win32_PerfFormattedData_Counters_ThermalZoneInformation")
        .ok()?;
    let celsius: Vec<f32> = rows
        .into_iter()
        .filter_map(|r| deci_kelvin_to_celsius(r.temperature))
        .collect();
    hottest(&celsius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deci_kelvin_zero_celsius() {
        // 273.15 K * 10 = 2731.5, rounded to 2732 — gives ~0.05 °C
        assert!((deci_kelvin_to_celsius(2732).unwrap() - 0.05).abs() < 0.1);
    }

    #[test]
    fn deci_kelvin_room_temp() {
        // 20 °C = 293.15 K = 2931.5 dK ≈ 2932
        assert!((deci_kelvin_to_celsius(2932).unwrap() - 20.0).abs() < 0.1);
    }

    #[test]
    fn deci_kelvin_typical_cpu_idle() {
        // 45 °C = 318.15 K = 3181.5 dK ≈ 3182
        assert!((deci_kelvin_to_celsius(3182).unwrap() - 45.0).abs() < 0.1);
    }

    #[test]
    fn deci_kelvin_rejects_absolute_zero() {
        // Sensor returning 0 dK is meaningless — well below 0 °C
        assert!(deci_kelvin_to_celsius(0).is_none());
    }

    #[test]
    fn deci_kelvin_rejects_too_hot() {
        // 200 °C — sensor is broken or value is bogus
        // 200 °C = 473.15 K = 4731.5 dK ≈ 4732
        assert!(deci_kelvin_to_celsius(4732).is_none());
    }

    #[test]
    fn deci_kelvin_accepts_realistic_cpu_high() {
        // 90 °C = 363.15 K = 3631.5 dK ≈ 3632
        assert!((deci_kelvin_to_celsius(3632).unwrap() - 90.0).abs() < 0.1);
    }

    #[test]
    fn hottest_empty() {
        assert_eq!(hottest(&[]), None);
    }

    #[test]
    fn hottest_single() {
        assert_eq!(hottest(&[42.0]), Some(42.0));
    }

    #[test]
    fn hottest_picks_max() {
        assert_eq!(hottest(&[42.0, 67.5, 50.0, 33.0]), Some(67.5));
    }

    #[test]
    fn hottest_ignores_nan() {
        let v = vec![f32::NAN, 40.0, 50.0];
        assert_eq!(hottest(&v), Some(50.0));
    }
}
