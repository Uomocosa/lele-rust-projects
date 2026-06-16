use std::sync::mpsc;
use std::time::Duration;

use crate::p2p::testing::Error;
use crate::p2p::testing::ProcessOrchestrator;

pub fn new(timeout: Duration) -> Result<ProcessOrchestrator, Error> {
    let exe = std::env::current_exe().map_err(Error::CurrentExe)?;
    let (tx, rx) = mpsc::channel();
    Ok(ProcessOrchestrator {
        exe,
        timeout,
        children: Vec::new(),
        _tx: tx,
        rx,
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::p2p::testing::ProcessOrchestrator;

    #[test]
    fn test_usage() {
        let orch = ProcessOrchestrator::new(Duration::from_secs(10)).unwrap();
        let peer_id_str = orch.exe.to_string_lossy().to_string();
        assert!(
            !peer_id_str.is_empty(),
            "Executable path should not be empty"
        );
        assert_eq!(orch.timeout, Duration::from_secs(10));
        assert!(orch.children.is_empty());
    }
}
