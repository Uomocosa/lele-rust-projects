use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;

use super::test_log_line;
use super::test_log_open;

pub struct TestLog {
    pub path: PathBuf,
    pub file: Mutex<Option<File>>,
}

#[rustfmt::skip]
impl TestLog {
    #[must_use]
    pub fn open(test_name: &str) -> Self { test_log_open::open(test_name) }
    pub fn line(&self, message: &str) { test_log_line::line(self, message) }
}

#[cfg(test)]
mod tests {
    use super::TestLog;

    #[test]
    fn test_usage() {
        let log = TestLog::open("test_log_delegate_smoke");
        log.line("delegate smoke");
        assert!(log.path.ends_with("test_log_delegate_smoke.log"));
        let _ = std::fs::remove_file(&log.path);
    }
}
