use crate::lhm_bridge::{GpuReading, SharedReading};
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct GpuStats {
    pub gpus: Vec<GpuReading>,
    pub sensor_available: bool,
}

pub fn read(reading: &SharedReading, available: bool) -> GpuStats {
    let guard = reading.lock().unwrap();
    match &*guard {
        Some(r) => GpuStats {
            gpus: r.gpus.clone(),
            sensor_available: available,
        },
        None => GpuStats {
            gpus: vec![],
            sensor_available: available,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lhm_bridge::{CpuReading, HardwareReading};
    use std::sync::{Arc, Mutex};

    #[test]
    fn empty_when_no_reading() {
        let r: SharedReading = Arc::new(Mutex::new(None));
        let stats = read(&r, true);
        assert!(stats.gpus.is_empty());
        assert!(stats.sensor_available);
    }

    #[test]
    fn forwards_gpus_from_reading() {
        let hr = HardwareReading {
            ts: 0,
            cpu: CpuReading::default(),
            gpus: vec![GpuReading {
                name: "Test GPU".into(),
                load_pct: Some(50.0),
                temp_c: Some(60.0),
                vram_used_mb: 1024,
                vram_total_mb: 8192,
            }],
        };
        let r: SharedReading = Arc::new(Mutex::new(Some(hr)));
        let stats = read(&r, true);
        assert_eq!(stats.gpus.len(), 1);
        assert_eq!(stats.gpus[0].name, "Test GPU");
    }

    #[test]
    fn marks_unavailable_when_flag_false() {
        let r: SharedReading = Arc::new(Mutex::new(None));
        let stats = read(&r, false);
        assert!(!stats.sensor_available);
    }
}
