use serde::Serialize;
use std::collections::HashMap;
use sysinfo::Disks;

#[derive(Debug, Clone, Default, Serialize)]
pub struct DiskEntry {
    pub name: String,
    pub mount: String,
    pub read_bps: u64,
    pub write_bps: u64,
    pub free: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DiskStats {
    pub disks: Vec<DiskEntry>,
}

pub struct DiskMonitor {
    disks: Disks,
    prev_io: HashMap<String, (u64, u64)>,
    last_poll: Option<std::time::Instant>,
}

impl DiskMonitor {
    pub fn new() -> Self {
        Self {
            disks: Disks::new_with_refreshed_list(),
            prev_io: HashMap::new(),
            last_poll: None,
        }
    }

    pub fn poll(&mut self) -> DiskStats {
        self.disks.refresh_list();
        let now = std::time::Instant::now();
        let interval_ms = self
            .last_poll
            .map(|t| now.duration_since(t).as_millis() as u64)
            .unwrap_or(1000);
        self.last_poll = Some(now);

        let io = read_disk_io_wmi().unwrap_or_default();

        let mut entries = Vec::new();
        for d in self.disks.iter() {
            let mount = d.mount_point().to_string_lossy().to_string();
            let name = mount
                .trim_end_matches('\\')
                .trim_end_matches('/')
                .to_string();

            let (read_bps, write_bps) = match (self.prev_io.get(&name), io.get(&name)) {
                (Some((pr, pw)), Some((cr, cw))) => (
                    delta_bps(*pr, *cr, interval_ms),
                    delta_bps(*pw, *cw, interval_ms),
                ),
                _ => (0, 0),
            };

            if let Some(cur) = io.get(&name) {
                self.prev_io.insert(name.clone(), *cur);
            }

            entries.push(DiskEntry {
                name,
                mount,
                read_bps,
                write_bps,
                free: d.available_space(),
                total: d.total_space(),
            });
        }
        DiskStats { disks: entries }
    }
}

pub fn delta_bps(prev: u64, curr: u64, interval_ms: u64) -> u64 {
    if curr < prev || interval_ms == 0 {
        return 0;
    }
    ((curr - prev) * 1000) / interval_ms
}

#[cfg(target_os = "windows")]
fn read_disk_io_wmi() -> Option<HashMap<String, (u64, u64)>> {
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct PerfDisk {
        name: String,
        disk_read_bytes_persec: u64,
        disk_write_bytes_persec: u64,
    }

    let com = COMLibrary::new().ok()?;
    let conn = WMIConnection::new(com).ok()?;
    let rows: Vec<PerfDisk> = conn
        .raw_query(
            "SELECT Name, DiskReadBytesPersec, DiskWriteBytesPersec FROM Win32_PerfRawData_PerfDisk_LogicalDisk",
        )
        .ok()?;
    let mut map = HashMap::new();
    for r in rows {
        if r.name == "_Total" || r.name.len() < 2 {
            continue;
        }
        map.insert(r.name, (r.disk_read_bytes_persec, r.disk_write_bytes_persec));
    }
    Some(map)
}

#[cfg(not(target_os = "windows"))]
fn read_disk_io_wmi() -> Option<HashMap<String, (u64, u64)>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_bps_basic() {
        assert_eq!(delta_bps(0, 1_000, 1000), 1_000);
    }

    #[test]
    fn delta_bps_counter_reset() {
        assert_eq!(delta_bps(100, 50, 1000), 0);
    }

    #[test]
    fn delta_bps_zero_interval() {
        assert_eq!(delta_bps(0, 1000, 0), 0);
    }

    #[test]
    fn delta_bps_half_second_doubles_rate() {
        assert_eq!(delta_bps(0, 500, 500), 1000);
    }

    #[test]
    fn delta_bps_equal_no_change() {
        assert_eq!(delta_bps(1000, 1000, 1000), 0);
    }

    #[test]
    fn poll_returns_at_least_one_disk_on_windows() {
        let mut m = DiskMonitor::new();
        let stats = m.poll();
        #[cfg(target_os = "windows")]
        assert!(!stats.disks.is_empty(), "Windows must have at least C:");
        let _ = stats;
    }
}
