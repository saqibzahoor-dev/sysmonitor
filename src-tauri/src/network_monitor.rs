use sysinfo::Networks;

pub struct NetworkMonitor {
    networks: Networks,
    prev_rx: u64,
    prev_tx: u64,
    initialized: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpeedData {
    pub download_bps: u64,
    pub upload_bps: u64,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            prev_rx: 0,
            prev_tx: 0,
            initialized: false,
        }
    }

    pub fn calculate_speed(prev: u64, current: u64, interval_ms: u64) -> u64 {
        if current < prev || interval_ms == 0 {
            return 0;
        }
        let delta = current - prev;
        (delta * 1000) / interval_ms
    }

    pub fn poll(&mut self) -> SpeedData {
        self.networks.refresh_list();

        let mut total_rx: u64 = 0;
        let mut total_tx: u64 = 0;

        for (_name, data) in self.networks.iter() {
            total_rx += data.total_received();
            total_tx += data.total_transmitted();
        }

        let speed = if !self.initialized {
            self.initialized = true;
            SpeedData {
                download_bps: 0,
                upload_bps: 0,
            }
        } else {
            SpeedData {
                download_bps: Self::calculate_speed(self.prev_rx, total_rx, 1000),
                upload_bps: Self::calculate_speed(self.prev_tx, total_tx, 1000),
            }
        };

        self.prev_rx = total_rx;
        self.prev_tx = total_tx;
        speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_speed_normal() {
        let speed = NetworkMonitor::calculate_speed(0, 1_000_000, 1000);
        assert_eq!(speed, 1_000_000);
    }

    #[test]
    fn test_calculate_speed_half_second() {
        let speed = NetworkMonitor::calculate_speed(0, 500_000, 500);
        assert_eq!(speed, 1_000_000);
    }

    #[test]
    fn test_calculate_speed_counter_reset() {
        let speed = NetworkMonitor::calculate_speed(1000, 500, 1000);
        assert_eq!(speed, 0);
    }

    #[test]
    fn test_calculate_speed_zero_interval() {
        let speed = NetworkMonitor::calculate_speed(0, 1000, 0);
        assert_eq!(speed, 0);
    }

    #[test]
    fn test_calculate_speed_no_change() {
        let speed = NetworkMonitor::calculate_speed(1000, 1000, 1000);
        assert_eq!(speed, 0);
    }
}
