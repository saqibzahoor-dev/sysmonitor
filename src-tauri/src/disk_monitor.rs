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

/// Sum read+write throughput across all drives. Useful for the compact bar
/// which shows total disk activity rather than per-drive numbers.
pub fn total_throughput_bps(disks: &[DiskEntry]) -> u64 {
    disks.iter().map(|d| d.read_bps + d.write_bps).sum()
}

/// Sum free space across all drives in bytes.
pub fn total_free_bytes(disks: &[DiskEntry]) -> u64 {
    disks.iter().map(|d| d.free).sum()
}

#[cfg(target_os = "windows")]
mod wmi_cache {
    use std::cell::RefCell;
    use wmi::{COMLibrary, WMIConnection};

    // Thread-local cached WMI connection. WMIConnection's COM raw pointer
    // isn't Send, but DiskMonitor::poll() always runs on the dedicated
    // monitor thread (and on the single test thread in cargo test), so a
    // thread-local cache is correct AND avoids fighting the Send trait.
    //
    // PERF: was opening a fresh COMLibrary + WMIConnection EVERY tick
    // (~50-150ms per call). Now opened once per thread, reused forever.
    thread_local! {
        static WMI: RefCell<Option<WMIConnection>> = const { RefCell::new(None) };
        static INIT_FAILED: RefCell<bool> = const { RefCell::new(false) };
    }

    pub fn with_wmi<F, T>(f: F) -> Option<T>
    where
        F: FnOnce(&WMIConnection) -> Option<T>,
    {
        // Lazy-init on first call.
        WMI.with(|cell| -> Option<T> {
            if cell.borrow().is_none() {
                if INIT_FAILED.with(|f| *f.borrow()) {
                    return None;
                }
                let init = (|| -> Option<WMIConnection> {
                    let com = COMLibrary::new().ok()?;
                    WMIConnection::new(com).ok()
                })();
                match init {
                    Some(c) => *cell.borrow_mut() = Some(c),
                    None => {
                        INIT_FAILED.with(|f| *f.borrow_mut() = true);
                        return None;
                    }
                }
            }
            let borrowed = cell.borrow();
            let conn = borrowed.as_ref()?;
            f(conn)
        })
    }

    /// Drop the cached connection so the next call re-initializes — used
    /// when a query fails to recover from transient errors.
    pub fn invalidate() {
        WMI.with(|cell| *cell.borrow_mut() = None);
    }
}

#[cfg(target_os = "windows")]
fn read_disk_io_wmi() -> Option<HashMap<String, (u64, u64)>> {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct PerfDisk {
        name: String,
        disk_read_bytes_persec: u64,
        disk_write_bytes_persec: u64,
    }

    wmi_cache::with_wmi(|conn| {
        let rows_result: Result<Vec<PerfDisk>, _> = conn.raw_query(
            "SELECT Name, DiskReadBytesPersec, DiskWriteBytesPersec FROM Win32_PerfRawData_PerfDisk_LogicalDisk",
        );
        match rows_result {
            Ok(rows) => {
                let mut map = HashMap::new();
                for r in rows {
                    if r.name == "_Total" || r.name.len() < 2 {
                        continue;
                    }
                    map.insert(r.name, (r.disk_read_bytes_persec, r.disk_write_bytes_persec));
                }
                Some(map)
            }
            Err(_) => {
                wmi_cache::invalidate();
                None
            }
        }
    })
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

    fn entry(read: u64, write: u64, free: u64, total: u64) -> DiskEntry {
        DiskEntry { name: "X".into(), mount: "X:\\".into(), read_bps: read, write_bps: write, free, total }
    }

    #[test]
    fn total_throughput_sums_read_plus_write() {
        let disks = vec![entry(1000, 500, 0, 0), entry(2000, 1500, 0, 0)];
        assert_eq!(total_throughput_bps(&disks), 5000);
    }

    #[test]
    fn total_throughput_empty_is_zero() {
        assert_eq!(total_throughput_bps(&[]), 0);
    }

    #[test]
    fn total_free_sums_across_drives() {
        let disks = vec![entry(0, 0, 100, 1000), entry(0, 0, 200, 2000)];
        assert_eq!(total_free_bytes(&disks), 300);
    }
}
