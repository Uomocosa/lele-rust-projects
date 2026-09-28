use std::process::Child;
use std::time::{Duration, Instant};

use crate::Error;

pub fn wait_until(
    child: &mut Child,
    what: &str,
    timeout: Duration,
    mut ready: impl FnMut() -> bool,
) -> Result<(), Error> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| Error::Timeout(what.to_string()))?;
    loop {
        if ready() {
            return Ok(());
        }
        if let Some(status) = child.try_wait()? {
            return Err(Error::Timeout(format!("{what} (process exited: {status})")));
        }
        if Instant::now() >= deadline {
            return Err(Error::Timeout(what.to_string()));
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use super::wait_until;
    use crate::process;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let command = vec!["sleep".to_string(), "5".to_string()];
        let mut child = process::spawn_app(
            &command,
            dir.path(),
            &BTreeMap::new(),
            &dir.path().join("l"),
        )
        .unwrap();
        let mut calls = 0;
        let result = wait_until(&mut child, "counter", Duration::from_secs(5), || {
            calls += 1;
            calls > 2
        });
        process::stop_app(&mut child);
        assert!(result.is_ok());
    }
}
