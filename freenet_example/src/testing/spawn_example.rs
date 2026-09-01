use std::process::Child;
use std::sync::mpsc::Receiver;

/// # Errors
/// Returns an error if the example fails to build or spawn.
pub fn spawn_example(example: &str, args: &[&str]) -> Result<(Child, Receiver<String>), String> {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let status = std::process::Command::new("cargo")
        .args(["build", "--example", example])
        .env("CARGO_TARGET_DIR", &target_dir)
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("cargo build failed: {e}"))?;
    if !status.success() {
        return Err(format!("cargo build failed for example {example}"));
    }
    let exe = std::path::Path::new(&target_dir)
        .join("debug")
        .join("examples")
        .join(example);
    let mut cmd = std::process::Command::new(exe);
    cmd.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| format!("spawn failed: {e}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "missing stdout".to_string())?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| "missing stderr".to_string())?;
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
    Ok((child, rx))
}

// no test_usage necessary
