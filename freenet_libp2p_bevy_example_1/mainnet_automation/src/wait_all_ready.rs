use std::time::Duration;

use crate::Error;
use crate::instance;
use crate::is_ready;

const POLL_MS: u64 = 5000;

pub fn wait_all_ready(instances: &[instance::Instance], timeout_secs: u64) -> Result<(), Error> {
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        let mut yes = 0usize;
        for inst in instances {
            if is_ready::is_ready(&inst.log_path)? {
                yes += 1;
            }
        }
        if yes == instances.len() {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err(Error::Assertion(format!(
                "timed out after {timeout_secs}s waiting for embedded node ready ({yes}/{})",
                instances.len()
            )));
        }
        std::thread::sleep(Duration::from_millis(POLL_MS));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::wait_all_ready;
    use crate::instance;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir().join(format!("ma_wait_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let log = dir.join("app.log");
        fs::write(&log, "still booting").unwrap();
        let inst = instance::Instance {
            index: 0,
            pid: 1,
            log_path: log,
            identity_dir: dir,
        };
        assert!(wait_all_ready(&[inst], 1).is_err());
    }
}
