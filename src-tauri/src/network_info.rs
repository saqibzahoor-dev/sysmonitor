use std::process::Command;

#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct NetworkInfo {
    pub ssid: String,
    pub connection_type: String,
    pub local_ip: String,
    pub public_ip: String,
    pub gateway: String,
    pub dns: Vec<String>,
    pub mac: String,
    pub signal_percent: Option<u32>,
}

pub struct NetworkInfoCollector {
    cached_public_ip: Option<String>,
    last_public_ip_fetch: std::time::Instant,
}

fn new_hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

impl NetworkInfoCollector {
    pub fn new() -> Self {
        Self {
            cached_public_ip: None,
            last_public_ip_fetch: std::time::Instant::now() - std::time::Duration::from_secs(600),
        }
    }

    pub fn collect(&mut self) -> NetworkInfo {
        let mut info = NetworkInfo::default();

        if let Ok(wifi) = Self::get_wifi_info() {
            info.ssid = wifi.0;
            info.signal_percent = wifi.1;
            info.connection_type = "WiFi".to_string();
        } else {
            info.connection_type = "Ethernet".to_string();
            info.ssid = "N/A".to_string();
        }

        if let Ok(ipconfig) = Self::get_ipconfig() {
            info.local_ip = ipconfig.0;
            info.gateway = ipconfig.1;
            info.dns = ipconfig.2;
            info.mac = ipconfig.3;
        }

        if self.last_public_ip_fetch.elapsed() > std::time::Duration::from_secs(300)
            || self.cached_public_ip.is_none()
        {
            info.public_ip = Self::fetch_public_ip();
            self.cached_public_ip = Some(info.public_ip.clone());
            self.last_public_ip_fetch = std::time::Instant::now();
        } else {
            info.public_ip = self.cached_public_ip.clone().unwrap_or_default();
        }

        info
    }

    fn get_wifi_info() -> Result<(String, Option<u32>), String> {
        let output = new_hidden_command("netsh")
            .args(["wlan", "show", "interfaces"])
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut ssid = String::new();
        let mut signal: Option<u32> = None;

        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("SSID") && !line.starts_with("BSSID") {
                if let Some(val) = line.split(':').nth(1) {
                    ssid = val.trim().to_string();
                }
            }
            if line.starts_with("Signal") {
                if let Some(val) = line.split(':').nth(1) {
                    let pct = val.trim().replace('%', "");
                    signal = pct.parse().ok();
                }
            }
        }

        if ssid.is_empty() {
            Err("no WiFi".to_string())
        } else {
            Ok((ssid, signal))
        }
    }

    fn get_ipconfig() -> Result<(String, String, Vec<String>, String), String> {
        let output = new_hidden_command("ipconfig")
            .args(["/all"])
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut ip = String::new();
        let mut gateway = String::new();
        let mut dns = Vec::new();
        let mut mac = String::new();
        let mut in_active_adapter = false;

        for line in stdout.lines() {
            let line_trimmed = line.trim();

            if (line.contains("Wireless") || line.contains("Ethernet") || line.contains("Wi-Fi"))
                && line.contains("adapter")
                && !line.contains("Virtual")
                && !line.contains("vEthernet")
                && !line.contains("Bluetooth")
            {
                in_active_adapter = true;
                continue;
            }
            if !line.starts_with(' ') && !line.starts_with('\t') && !line_trimmed.is_empty() {
                if in_active_adapter && !ip.is_empty() {
                    break;
                }
                in_active_adapter = false;
            }

            if !in_active_adapter {
                continue;
            }

            if line_trimmed.contains("IPv4 Address") || line_trimmed.contains("IPv4") {
                if let Some(val) = line_trimmed.split(':').last() {
                    ip = val
                        .trim()
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .trim()
                        .to_string();
                }
            }
            if line_trimmed.contains("Default Gateway") && !line_trimmed.ends_with(':') {
                if let Some(val) = line_trimmed.split(':').last() {
                    let gw = val.trim().to_string();
                    if !gw.is_empty() {
                        gateway = gw;
                    }
                }
            }
            if line_trimmed.contains("DNS Servers") || line_trimmed.contains("DNS") {
                if let Some(val) = line_trimmed.split(':').last() {
                    let d = val.trim().to_string();
                    if !d.is_empty()
                        && d.contains('.')
                        && !d.contains("DNS")
                        && dns.len() < 4
                    {
                        dns.push(d);
                    }
                }
            }
            if line_trimmed.contains("Physical Address") {
                if let Some(colon_pos) = line_trimmed.find(": ") {
                    mac = line_trimmed[colon_pos + 2..].trim().to_string();
                }
            }
        }

        if ip.is_empty() {
            Err("no active adapter".to_string())
        } else {
            Ok((ip, gateway, dns, mac))
        }
    }

    fn fetch_public_ip() -> String {
        let urls = [
            "https://api.ipify.org",
            "https://api.my-ip.io/v2/ip.txt",
            "https://ifconfig.me/ip",
        ];

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build();

        let client = match client {
            Ok(c) => c,
            Err(_) => return "unavailable".to_string(),
        };

        for url in &urls {
            if let Ok(resp) = client.get(*url).send() {
                if let Ok(text) = resp.text() {
                    let ip = text.trim().to_string();
                    if !ip.is_empty() && ip.len() < 50 {
                        return ip;
                    }
                }
            }
        }
        "unavailable".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collector() {
        let collector = NetworkInfoCollector::new();
        assert!(collector.cached_public_ip.is_none());
    }

    #[test]
    fn test_network_info_default() {
        let info = NetworkInfo::default();
        assert!(info.ssid.is_empty());
        assert!(info.local_ip.is_empty());
        assert!(info.signal_percent.is_none());
    }
}
