use std::sync::mpsc::Receiver;
use std::time::Duration;

/// # Errors
/// Returns an error if the deadline overflows or the prefix is not found in time.
pub fn expect_line(
    rx: &Receiver<String>,
    prefix: &str,
    timeout: Duration,
) -> Result<String, String> {
    let Some(deadline) = std::time::Instant::now().checked_add(timeout) else {
        return Err(format!("deadline overflow for timeout {timeout:?}"));
    };
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(line) => {
                if line.starts_with(prefix) {
                    return Ok(line);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if std::time::Instant::now() >= deadline {
                    return Err(format!(
                        "timed out waiting for line starting with: {prefix}"
                    ));
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return Err(format!("subprocess exited before printing: {prefix}"));
            }
        }
    }
}

// no test_usage necessary
