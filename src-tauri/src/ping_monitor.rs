use std::net::Ipv4Addr;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PingResult {
    pub latency_ms: Option<u32>,
    pub status: PingStatus,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub enum PingStatus {
    Ok,
    Timeout,
    Error,
}

pub struct PingMonitor {
    target: Ipv4Addr,
}

impl PingMonitor {
    pub fn new() -> Self {
        Self {
            target: Ipv4Addr::new(8, 8, 8, 8),
        }
    }

    pub fn ping(&self) -> PingResult {
        match self.ping_impl() {
            Ok(ms) => PingResult {
                latency_ms: Some(ms),
                status: PingStatus::Ok,
            },
            Err(e) => {
                if e.contains("timeout") {
                    PingResult {
                        latency_ms: None,
                        status: PingStatus::Timeout,
                    }
                } else {
                    PingResult {
                        latency_ms: None,
                        status: PingStatus::Error,
                    }
                }
            }
        }
    }

    fn ping_impl(&self) -> Result<u32, String> {
        let mut cmd = std::process::Command::new("ping");
        cmd.args(["-n", "1", "-w", "2000", &self.target.to_string()]);

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let output = cmd.output().map_err(|e| format!("ping failed: {}", e))?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.contains("Request timed out")
            || stdout.contains("unreachable")
            || stdout.contains("could not find host")
        {
            return Err("timeout".to_string());
        }

        // Parse "time=12ms" or "time<1ms"
        if let Some(time_pos) = stdout.find("time=") {
            let after = &stdout[time_pos + 5..];
            let ms_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            ms_str.parse::<u32>().map_err(|e| e.to_string())
        } else if stdout.contains("time<1ms") || stdout.contains("time<1") {
            Ok(0)
        } else {
            Err("could not parse ping output".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_result_ok() {
        let result = PingResult {
            latency_ms: Some(12),
            status: PingStatus::Ok,
        };
        assert_eq!(result.latency_ms, Some(12));
    }

    #[test]
    fn test_ping_result_timeout() {
        let result = PingResult {
            latency_ms: None,
            status: PingStatus::Timeout,
        };
        assert!(result.latency_ms.is_none());
        assert_eq!(result.status, PingStatus::Timeout);
    }

    #[test]
    fn test_default_target() {
        let monitor = PingMonitor::new();
        assert_eq!(monitor.target, Ipv4Addr::new(8, 8, 8, 8));
    }
}
