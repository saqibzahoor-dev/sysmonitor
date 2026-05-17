use serde::Serialize;
use sysinfo::{MemoryRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Default, Serialize)]
pub struct MemStats {
    pub used: u64,
    pub total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
}

pub struct MemoryMonitor {
    sys: System,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        Self {
            sys: System::new_with_specifics(
                RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
            ),
        }
    }

    pub fn poll(&mut self) -> MemStats {
        self.sys.refresh_memory();
        MemStats {
            used: self.sys.used_memory(),
            total: self.sys.total_memory(),
            swap_used: self.sys.used_swap(),
            swap_total: self.sys.total_swap(),
        }
    }
}

pub fn percent_used(used: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64 * 100.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_used_zero_total() {
        assert_eq!(percent_used(0, 0), 0.0);
        assert_eq!(percent_used(100, 0), 0.0);
    }

    #[test]
    fn percent_used_half() {
        assert!((percent_used(500, 1000) - 50.0).abs() < 0.001);
    }

    #[test]
    fn percent_used_full() {
        assert!((percent_used(1000, 1000) - 100.0).abs() < 0.001);
    }

    #[test]
    fn poll_returns_nonzero_total() {
        let mut m = MemoryMonitor::new();
        let stats = m.poll();
        assert!(stats.total > 0, "system must report some total RAM");
        assert!(stats.used <= stats.total);
    }

    #[test]
    fn stats_serialize() {
        let s = MemStats { used: 1024, total: 8192, swap_used: 0, swap_total: 4096 };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"used\":1024"));
        assert!(json.contains("\"swap_total\":4096"));
    }
}
