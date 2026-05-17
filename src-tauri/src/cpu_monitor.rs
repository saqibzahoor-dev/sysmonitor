use serde::Serialize;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Default, Serialize)]
pub struct CpuStats {
    pub usage: f32,
    pub per_core: Vec<f32>,
    pub freq_mhz: u64,
}

pub struct CpuMonitor {
    sys: System,
}

impl CpuMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
        );
        sys.refresh_cpu_all();
        Self { sys }
    }

    pub fn poll(&mut self) -> CpuStats {
        self.sys.refresh_cpu_all();
        let cpus = self.sys.cpus();
        let per_core: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
        let usage = average(&per_core);
        let freq_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);
        CpuStats { usage, per_core, freq_mhz }
    }
}

pub fn average(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_of_empty_is_zero() {
        assert_eq!(average(&[]), 0.0);
    }

    #[test]
    fn average_computes_mean() {
        assert!((average(&[10.0, 20.0, 30.0]) - 20.0).abs() < 0.001);
    }

    #[test]
    fn average_of_single_value() {
        assert!((average(&[42.5]) - 42.5).abs() < 0.001);
    }

    #[test]
    fn stats_serialize_to_json() {
        let stats = CpuStats { usage: 12.5, per_core: vec![10.0, 15.0], freq_mhz: 4500 };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"usage\":12.5"));
        assert!(json.contains("\"freq_mhz\":4500"));
    }

    #[test]
    fn poll_returns_nonempty_per_core() {
        let mut m = CpuMonitor::new();
        std::thread::sleep(std::time::Duration::from_millis(250));
        let stats = m.poll();
        assert!(!stats.per_core.is_empty(), "must have at least one logical core");
        assert!(stats.usage >= 0.0 && stats.usage <= 100.0);
    }
}
