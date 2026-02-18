use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub struct FileState {
    pub filename: String,
    pub workspace: String,
    pub start_time: Instant,
}

impl FileState {
    pub fn new(filename: String, workspace: String) -> Self {
        Self {
            filename,
            workspace,
            start_time: Instant::now(),
        }
    }

    pub fn get_start_timestamp(&self) -> u64 {
        let now = SystemTime::now();
        let elapsed = self.start_time.elapsed();
        now.duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(elapsed.as_secs())
    }
}

pub struct WorkspaceState {
    pub workspace: String,
    pub start_time: Instant,
}

impl WorkspaceState {
    pub fn new(workspace: String) -> Self {
        Self {
            workspace,
            start_time: Instant::now(),
        }
    }

    pub fn get_start_timestamp(&self) -> u64 {
        let now = SystemTime::now();
        let elapsed = self.start_time.elapsed();
        now.duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(elapsed.as_secs())
    }
}
