use std::time::Instant;

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
}
