use std::time::Duration;

use crate::Error;
use crate::applied_player_ids;
use crate::instance;

const POLL_MS: u64 = 5000;

pub fn wait_all_converged(
    instances: &[instance::Instance],
    timeout_secs: u64,
) -> Result<(), Error> {
    let n = instances.len();
    if n == 0 {
        return Ok(());
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        let mut unconverged = 0usize;
        for inst in instances {
            let peers = applied_player_ids::applied_player_ids(&inst.log_path)?;
            if peers.len() < n - 1 {
                unconverged += 1;
            }
        }
        if unconverged == 0 {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            let mut detail = String::new();
            for inst in instances {
                let peers = applied_player_ids::applied_player_ids(&inst.log_path)?;
                detail.push_str(&format!(
                    "  instance-{} peers={}/{} log={}\n",
                    inst.index,
                    peers.len(),
                    n - 1,
                    inst.log_path.display()
                ));
            }
            return Err(Error::Assertion(format!(
                "timed out after {timeout_secs}s waiting for {} instances to mutually converge \
                 (each should see {} peers):\n{detail}",
                n,
                n - 1
            )));
        }
        std::thread::sleep(Duration::from_millis(POLL_MS));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::wait_all_converged;
    use crate::instance;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir().join(format!("ma_conv_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let log = dir.join("app.log");
        fs::write(&log, "no peers yet").unwrap();
        let a = instance::Instance {
            index: 0,
            pid: 1,
            log_path: log.clone(),
            identity_dir: dir.clone(),
        };
        let b = instance::Instance {
            index: 1,
            pid: 2,
            log_path: log,
            identity_dir: dir,
        };
        assert!(wait_all_converged(&[a, b], 1).is_err());
    }
}
