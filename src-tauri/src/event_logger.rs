use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone, serde::Serialize)]
pub struct NetworkEvent {
    pub timestamp: String,
    pub message: String,
}

pub struct EventLogger {
    events: VecDeque<NetworkEvent>,
    start_time: Instant,
    last_download_bps: u64,
    last_ip: String,
    last_interface: String,
}

impl EventLogger {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            start_time: Instant::now(),
            last_download_bps: 0,
            last_ip: String::new(),
            last_interface: String::new(),
        }
    }

    pub fn check_events(&mut self, download_bps: u64, ip: &str, interface: &str) {
        // Speed drop > 50%
        if self.last_download_bps > 0 && download_bps > 0 {
            let threshold = self.last_download_bps / 2;
            if download_bps < threshold {
                self.log("speed drop detected");
            }
        }

        // IP change
        if !self.last_ip.is_empty() && ip != self.last_ip && !ip.is_empty() {
            self.log(&format!("IP changed: {} -> {}", self.last_ip, ip));
        }

        // Interface switch
        if !self.last_interface.is_empty()
            && interface != self.last_interface
            && !interface.is_empty()
        {
            self.log(&format!("interface switched to {}", interface));
        }

        self.last_download_bps = download_bps;
        if !ip.is_empty() {
            self.last_ip = ip.to_string();
        }
        if !interface.is_empty() {
            self.last_interface = interface.to_string();
        }
    }

    fn log(&mut self, message: &str) {
        let elapsed = self.start_time.elapsed().as_secs();
        let timestamp = Self::format_duration(elapsed);

        self.events.push_back(NetworkEvent {
            timestamp,
            message: message.to_string(),
        });

        while self.events.len() > 50 {
            self.events.pop_front();
        }
    }

    pub fn get_events(&self) -> Vec<NetworkEvent> {
        self.events.iter().cloned().collect()
    }

    pub fn format_duration(secs: u64) -> String {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(EventLogger::format_duration(0), "00:00:00");
        assert_eq!(EventLogger::format_duration(61), "00:01:01");
        assert_eq!(EventLogger::format_duration(3661), "01:01:01");
    }

    #[test]
    fn test_speed_drop_detection() {
        let mut logger = EventLogger::new();
        logger.check_events(10_000, "192.168.1.1", "WiFi");
        logger.check_events(3_000, "192.168.1.1", "WiFi");
        assert_eq!(logger.events.len(), 1);
        assert!(logger.events[0].message.contains("speed drop"));
    }

    #[test]
    fn test_no_false_speed_drop() {
        let mut logger = EventLogger::new();
        logger.check_events(10_000, "192.168.1.1", "WiFi");
        logger.check_events(6_000, "192.168.1.1", "WiFi");
        assert_eq!(logger.events.len(), 0);
    }

    #[test]
    fn test_ip_change_detection() {
        let mut logger = EventLogger::new();
        logger.check_events(1000, "192.168.1.1", "WiFi");
        logger.check_events(1000, "192.168.1.50", "WiFi");
        assert_eq!(logger.events.len(), 1);
        assert!(logger.events[0].message.contains("IP changed"));
    }

    #[test]
    fn test_interface_switch_detection() {
        let mut logger = EventLogger::new();
        logger.check_events(1000, "10.0.0.1", "Ethernet");
        logger.check_events(1000, "10.0.0.1", "WiFi");
        assert_eq!(logger.events.len(), 1);
        assert!(logger.events[0].message.contains("interface switched"));
    }

    #[test]
    fn test_event_cap_at_50() {
        let mut logger = EventLogger::new();
        for i in 0..60 {
            logger.log(&format!("event {}", i));
        }
        assert_eq!(logger.events.len(), 50);
    }
}
