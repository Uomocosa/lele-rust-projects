use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::p2p::testing::Output;
use crate::p2p::testing::ProcessOrchestrator;

pub fn collect(mut orch: ProcessOrchestrator) -> Output {
    let deadline = Instant::now() + orch.timeout;
    let mut lines = Vec::new();

    while Instant::now() < deadline {
        match orch.rx.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => {
                println!("[child] {}", line);
                lines.push(line);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let children = std::mem::take(&mut orch.children);
    for mut peer in children {
        if let Some(ref mut child) = peer.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        for h in peer.handles {
            let _ = h.join();
        }
    }

    while let Ok(line) = orch.rx.try_recv() {
        println!("[child] {}", line);
        lines.push(line);
    }

    Output { lines }
}
