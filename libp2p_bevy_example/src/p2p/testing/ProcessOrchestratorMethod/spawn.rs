use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;

use crate::p2p::testing::Error;
use crate::p2p::testing::ProcessOrchestrator;
use crate::p2p::testing::SpawnedPeer;

pub fn spawn(orch: &mut ProcessOrchestrator, tag: &str) -> Result<(), Error> {
    let mut child = Command::new(&orch.exe)
        .env("ORCH_PEER", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(Error::SpawnChild)?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let tx_out = orch._tx.clone();
    let tag_out = tag.to_string();
    let h_out = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let _ = tx_out.send(format!("{}:{}", tag_out, l));
                }
                Err(_) => return,
            }
        }
    });

    let tx_err = orch._tx.clone();
    let tag_err = format!("{}:stderr", tag);
    let h_err = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let _ = tx_err.send(format!("{}:{}", tag_err, l));
                }
                Err(_) => return,
            }
        }
    });

    orch.children.push(SpawnedPeer {
        child: Some(child),
        handles: vec![h_out, h_err],
    });

    Ok(())
}
