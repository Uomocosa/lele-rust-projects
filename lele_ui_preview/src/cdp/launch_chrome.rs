use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::Error;
use crate::cdp;

pub fn launch_chrome(executable: &str, profile_dir: &Path) -> Result<cdp::Browser, Error> {
    let mut child = Command::new(executable)
        .args([
            "--headless=new",
            "--remote-debugging-port=0",
            "--no-first-run",
            "--no-default-browser-check",
            "--disable-extensions",
            "--disable-background-networking",
            "--hide-scrollbars",
            "--mute-audio",
            "--force-device-scale-factor=1",
            "--force-color-profile=srgb",
            "--font-render-hinting=none",
        ])
        .arg(format!("--user-data-dir={}", profile_dir.display()))
        .arg("about:blank")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::Cdp(format!("cannot start {executable}: {e}")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::Cdp("chrome stderr unavailable".to_string()))?;
    let mut lines = BufReader::new(stderr).lines();
    let mut port = None;
    for line in lines.by_ref() {
        let line = line?;
        if let Some(found) = devtools_port(&line) {
            port = Some(found);
            break;
        }
    }
    std::thread::spawn(move || lines.for_each(drop));
    if let Some(port) = port {
        return Ok(cdp::Browser { child, port });
    }
    let _ = child.kill();
    let _ = child.wait();
    Err(Error::Cdp(
        "chrome never printed its DevTools port".to_string(),
    ))
}

// needed helper: extract the port from "DevTools listening on ws://127.0.0.1:PORT/..."
fn devtools_port(line: &str) -> Option<u16> {
    let rest = line.split("DevTools listening on ws://").nth(1)?;
    let host_port = rest.split('/').next()?;
    host_port.rsplit(':').next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::devtools_port;

    #[test]
    fn test_usage() {
        let line = "DevTools listening on ws://127.0.0.1:40123/devtools/browser/abc";
        assert_eq!(devtools_port(line), Some(40123));
        assert_eq!(devtools_port("something else"), None);
    }
}
