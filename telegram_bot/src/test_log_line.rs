use std::io::Write as _;

use super::test_log::TestLog;

pub fn line(log: &TestLog, message: &str) {
    let stamped = format!("{:?} {message}", std::time::SystemTime::now());
    write_line(log, &stamped);
    eprintln!("{stamped}");
}

// needed helper: appends one stamped line when the log file is available
fn write_line(log: &TestLog, stamped: &str) {
    let mut slot = log
        .file
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(file) = slot.as_mut() {
        let _ = writeln!(file, "{stamped}");
    }
}

#[cfg(test)]
mod tests {
    use crate::test_log::TestLog;

    use super::line;

    #[test]
    fn test_usage() {
        let log = TestLog::open("test_log_line_smoke");
        line(&log, "line smoke");
        let body = std::fs::read_to_string(&log.path).unwrap_or_default();
        assert!(body.contains("line smoke"));
        let _ = std::fs::remove_file(&log.path);
    }
}
