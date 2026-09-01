use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn spawn_binary(name: &str, args: &[&str]) -> (Child, Vec<mpsc::Receiver<String>>) {
    let mut cmd = Command::new(name);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("failed to spawn binary");

    let stdout = child.stdout.take().expect("no stdout");
    let stderr = child.stderr.take().expect("no stderr");

    let (stdout_tx, stdout_rx) = mpsc::channel::<String>();
    let (stderr_tx, stderr_rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                if stdout_tx.send(line).is_err() {
                    break;
                }
            }
        }
    });

    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                if stderr_tx.send(line).is_err() {
                    break;
                }
            }
        }
    });

    (child, vec![stdout_rx, stderr_rx])
}

pub fn expect_line(rx: &mpsc::Receiver<String>, timeout: Duration, contains: &str) -> String {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(line) => {
                if line.contains(contains) {
                    return line;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if std::time::Instant::now() > deadline {
                    panic!("timeout waiting for line containing: {contains}");
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("channel disconnected while waiting for: {contains}");
            }
        }
    }
}
