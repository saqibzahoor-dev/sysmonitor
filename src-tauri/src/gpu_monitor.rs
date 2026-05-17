use crate::lhm_bridge::{GpuReading, SharedReading};
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct GpuStats {
    pub gpus: Vec<GpuReading>,
    pub sensor_available: bool,
}

pub fn read(reading: &SharedReading, available: bool) -> GpuStats {
    let guard = reading.lock().unwrap();
    let mut gpus = match &*guard {
        Some(r) => r.gpus.clone(),
        None => vec![],
    };
    // Sort so the most-data-rich GPU comes first (discrete with temp > iGPU).
    // The compact bar shows gpus[0] — this ensures it shows the GPU with
    // real sensor readings (typically the NVIDIA / AMD discrete) rather than
    // an integrated GPU that LHM can't read.
    gpus.sort_by(|a, b| score_gpu(b).cmp(&score_gpu(a)));
    GpuStats { gpus, sensor_available: available }
}

/// Score a GPU reading by how much data it has — higher is better.
/// A GPU with both temperature AND load wins; null-everywhere loses.
pub fn score_gpu(g: &GpuReading) -> u32 {
    let mut score = 0;
    if g.temp_c.is_some() { score += 4; }
    if g.load_pct.is_some() { score += 2; }
    if g.vram_total_mb > 0 { score += 1; }
    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lhm_bridge::{CpuReading, HardwareReading};
    use std::sync::{Arc, Mutex};

    fn gpu(name: &str, load: Option<f32>, temp: Option<f32>, vram_total: u32) -> GpuReading {
        GpuReading {
            name: name.into(),
            load_pct: load,
            temp_c: temp,
            vram_used_mb: 0,
            vram_total_mb: vram_total,
        }
    }

    #[test]
    fn score_full_data() {
        assert_eq!(score_gpu(&gpu("X", Some(50.0), Some(70.0), 8192)), 7);
    }

    #[test]
    fn score_temp_only() {
        assert_eq!(score_gpu(&gpu("X", None, Some(70.0), 0)), 4);
    }

    #[test]
    fn score_null_everywhere() {
        assert_eq!(score_gpu(&gpu("AMD Radeon iGPU", None, None, 0)), 0);
    }

    #[test]
    fn discrete_with_temp_outranks_igpu_with_nulls() {
        let igpu = gpu("AMD Radeon iGPU", None, None, 0);
        let dgpu = gpu("NVIDIA Quadro", Some(27.0), Some(66.0), 2048);
        assert!(score_gpu(&dgpu) > score_gpu(&igpu));
    }

    #[test]
    fn read_sorts_discrete_first() {
        let hr = HardwareReading {
            ts: 0,
            cpu: CpuReading::default(),
            gpus: vec![
                gpu("AMD Radeon iGPU", None, None, 0),
                gpu("NVIDIA Quadro K2000", Some(27.0), Some(66.0), 2048),
            ],
        };
        let r: SharedReading = Arc::new(Mutex::new(Some(hr)));
        let stats = read(&r, true);
        assert_eq!(stats.gpus.len(), 2);
        assert_eq!(stats.gpus[0].name, "NVIDIA Quadro K2000");
        assert_eq!(stats.gpus[1].name, "AMD Radeon iGPU");
    }

    #[test]
    fn empty_when_no_reading() {
        let r: SharedReading = Arc::new(Mutex::new(None));
        let stats = read(&r, true);
        assert!(stats.gpus.is_empty());
    }

    #[test]
    fn read_forwards_single_gpu() {
        let hr = HardwareReading {
            ts: 0,
            cpu: CpuReading::default(),
            gpus: vec![gpu("Solo GPU", Some(50.0), Some(60.0), 8192)],
        };
        let r: SharedReading = Arc::new(Mutex::new(Some(hr)));
        let stats = read(&r, true);
        assert_eq!(stats.gpus.len(), 1);
        assert_eq!(stats.gpus[0].name, "Solo GPU");
    }

    #[test]
    fn marks_unavailable_when_flag_false() {
        let r: SharedReading = Arc::new(Mutex::new(None));
        let stats = read(&r, false);
        assert!(!stats.sensor_available);
    }
}

