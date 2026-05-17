use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProcInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_pct: f32,
    pub mem_bytes: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProcStats {
    pub top: Vec<ProcInfo>,
    pub count: usize,
    pub uptime_secs: u64,
}

pub struct ProcessMonitor {
    sys: System,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
        );
        sys.refresh_processes(ProcessesToUpdate::All, true);
        Self { sys }
    }

    pub fn poll(&mut self, top_n: usize) -> ProcStats {
        self.sys.refresh_processes(ProcessesToUpdate::All, true);
        let mut all: Vec<ProcInfo> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, p)| ProcInfo {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().to_string(),
                cpu_pct: p.cpu_usage(),
                mem_bytes: p.memory(),
            })
            .collect();

        let count = all.len();
        sort_by_cpu_desc(&mut all);
        all.truncate(top_n);

        ProcStats {
            top: all,
            count,
            uptime_secs: System::uptime(),
        }
    }
}

pub fn sort_by_cpu_desc(v: &mut [ProcInfo]) {
    v.sort_by(|a, b| {
        b.cpu_pct
            .partial_cmp(&a.cpu_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.mem_bytes.cmp(&a.mem_bytes))
    });
}

pub fn sort_by_mem_desc(v: &mut [ProcInfo]) {
    v.sort_by(|a, b| {
        b.mem_bytes
            .cmp(&a.mem_bytes)
            .then_with(|| {
                b.cpu_pct
                    .partial_cmp(&a.cpu_pct)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(pid: u32, name: &str, cpu: f32, mem: u64) -> ProcInfo {
        ProcInfo {
            pid,
            name: name.into(),
            cpu_pct: cpu,
            mem_bytes: mem,
        }
    }

    #[test]
    fn sort_by_cpu_desc_orders_highest_first() {
        let mut v = vec![p(1, "a", 10.0, 100), p(2, "b", 50.0, 50), p(3, "c", 20.0, 200)];
        sort_by_cpu_desc(&mut v);
        assert_eq!(v[0].pid, 2);
        assert_eq!(v[1].pid, 3);
        assert_eq!(v[2].pid, 1);
    }

    #[test]
    fn sort_by_cpu_desc_breaks_tie_by_mem() {
        let mut v = vec![p(1, "a", 5.0, 100), p(2, "b", 5.0, 500)];
        sort_by_cpu_desc(&mut v);
        assert_eq!(v[0].pid, 2, "tie on cpu, higher mem wins");
    }

    #[test]
    fn sort_by_mem_desc_orders_highest_first() {
        let mut v = vec![p(1, "a", 10.0, 100), p(2, "b", 50.0, 50), p(3, "c", 20.0, 200)];
        sort_by_mem_desc(&mut v);
        assert_eq!(v[0].pid, 3);
        assert_eq!(v[1].pid, 1);
        assert_eq!(v[2].pid, 2);
    }

    #[test]
    fn poll_returns_top_n_capped() {
        let mut m = ProcessMonitor::new();
        std::thread::sleep(std::time::Duration::from_millis(250));
        let stats = m.poll(5);
        assert!(stats.top.len() <= 5);
        assert!(stats.count >= stats.top.len());
        assert!(stats.uptime_secs > 0);
    }
}
