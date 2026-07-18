use std::time::Duration;

fn spawn_example(args: &[&str]) -> (std::process::Child, std::sync::mpsc::Receiver<String>) {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let _ = std::process::Command::new("cargo")
        .args(["build", "--example", "p2p_counter_gateway"])
        .env("CARGO_TARGET_DIR", &target_dir)
        .stderr(std::process::Stdio::null())
        .status();
    let exe = std::path::Path::new(&target_dir)
        .join("debug")
        .join("examples")
        .join("p2p_counter_gateway");
    let mut cmd = std::process::Command::new(exe);
    cmd.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf);
        if !buf.is_empty() {
            eprintln!("[subprocess stderr]: {}", String::from_utf8_lossy(&buf));
        }
    });
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::io::BufRead;
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if tx.send(l).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    (child, rx)
}

fn expect_line(rx: &std::sync::mpsc::Receiver<String>, prefix: &str, timeout: Duration) -> String {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(line) => {
                if line.starts_with(prefix) {
                    return line;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if std::time::Instant::now() >= deadline {
                    panic!("timed out waiting for line starting with: {prefix}");
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                panic!("subprocess exited before printing: {prefix}");
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_gateway_subprocess_smoke() {
    let (mut gateway, gw_rx) = spawn_example(&["--gateway", "--public-address", "127.0.0.1"]);

    let connect_line = expect_line(&gw_rx, "GATEWAY_CONNECT=", Duration::from_secs(30));
    assert!(
        connect_line.len() > "GATEWAY_CONNECT=".len(),
        "connect string should contain pubkey"
    );

    let deployed_line = expect_line(
        &gw_rx,
        "counter deployed, initial count:",
        Duration::from_secs(15),
    );
    let initial_count = deployed_line
        .split("initial count:")
        .nth(1)
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(u64::MAX);
    assert_eq!(initial_count, 0);

    let tick_line = expect_line(&gw_rx, "tick 1:", Duration::from_secs(10));
    let tick_count = tick_line
        .split("count =")
        .nth(1)
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    assert!(tick_count >= 1, "gateway should increment the counter");

    let _ = gateway.kill();
}
