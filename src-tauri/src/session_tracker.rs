use std::time::Instant;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionStats {
    pub duration_secs: u64,
    pub total_downloaded: u64,
    pub total_uploaded: u64,
    pub peak_download_bps: u64,
    pub peak_upload_bps: u64,
    pub active_connections: u32,
}

pub struct SessionTracker {
    start_time: Instant,
    total_downloaded: u64,
    total_uploaded: u64,
    peak_download: u64,
    peak_upload: u64,
}

impl SessionTracker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            total_downloaded: 0,
            total_uploaded: 0,
            peak_download: 0,
            peak_upload: 0,
        }
    }

    pub fn update(&mut self, download_bps: u64, upload_bps: u64) {
        self.total_downloaded += download_bps;
        self.total_uploaded += upload_bps;

        if download_bps > self.peak_download {
            self.peak_download = download_bps;
        }
        if upload_bps > self.peak_upload {
            self.peak_upload = upload_bps;
        }
    }

    pub fn get_stats(&self, active_connections: u32) -> SessionStats {
        SessionStats {
            duration_secs: self.start_time.elapsed().as_secs(),
            total_downloaded: self.total_downloaded,
            total_uploaded: self.total_uploaded,
            peak_download_bps: self.peak_download,
            peak_upload_bps: self.peak_upload,
            active_connections,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session() {
        let tracker = SessionTracker::new();
        let stats = tracker.get_stats(0);
        assert_eq!(stats.total_downloaded, 0);
        assert_eq!(stats.total_uploaded, 0);
        assert_eq!(stats.peak_download_bps, 0);
    }

    #[test]
    fn test_accumulate_bytes() {
        let mut tracker = SessionTracker::new();
        tracker.update(1_000_000, 500_000);
        tracker.update(2_000_000, 300_000);
        let stats = tracker.get_stats(5);
        assert_eq!(stats.total_downloaded, 3_000_000);
        assert_eq!(stats.total_uploaded, 800_000);
        assert_eq!(stats.active_connections, 5);
    }

    #[test]
    fn test_peak_tracking() {
        let mut tracker = SessionTracker::new();
        tracker.update(1_000, 500);
        tracker.update(5_000, 200);
        tracker.update(2_000, 800);
        let stats = tracker.get_stats(0);
        assert_eq!(stats.peak_download_bps, 5_000);
        assert_eq!(stats.peak_upload_bps, 800);
    }
}
