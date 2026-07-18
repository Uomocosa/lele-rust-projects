use std::io::BufRead;
use std::io::BufReader;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn strip_ansi(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_escape = false;
    for ch in line.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn build_release_binary() -> String {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "../target".into());
    let path = std::path::Path::new(&target_dir)
        .join("release")
        .join("freenet-example");
    assert!(
        path.exists(),
        "Release binary not found at {} — run `cargo build --release` first",
        path.display()
    );
    path.to_string_lossy().into_owned()
}

pub fn spawn_binary(path: &str, args: &[&str]) -> (Child, mpsc::Receiver<String>) {
    let mut cmd = Command::new(path);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = std::io::Read::read_to_end(&mut stderr, &mut buf);
        if !buf.is_empty() {
            eprintln!(
                "[binary stderr]: {}",
                String::from_utf8_lossy(&buf)
            );
        }
    });
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let clean = strip_ansi(&l);
                    if tx.send(clean).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    (child, rx)
}

pub fn expect_line(rx: &mpsc::Receiver<String>, contains: &str, timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(line) => {
                if line.contains(contains) {
                    return line;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if Instant::now() >= deadline {
                    panic!("timed out waiting for: {contains}");
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("binary exited before printing: {contains}");
            }
        }
    }
}
