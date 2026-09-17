use std::sync::Mutex;

use super::test_log::TestLog;

#[must_use]
pub fn open(test_name: &str) -> TestLog {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".local-run");
    let _ = std::fs::create_dir_all(&path);
    path.push(format!("{test_name}.log"));
    let file = std::fs::File::create(&path).ok();
    TestLog {
        path,
        file: Mutex::new(file),
    }
}

#[cfg(test)]
mod tests {
    use super::open;

    #[test]
    fn test_usage() {
        let log = open("test_log_open_smoke");
        assert!(log.path.ends_with("test_log_open_smoke.log"));
        let _ = std::fs::remove_file(&log.path);
    }
}
