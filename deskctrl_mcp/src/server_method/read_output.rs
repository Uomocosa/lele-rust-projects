use std::sync::atomic::Ordering;

use rmcp::model::{CallToolResult, ContentBlock};

use crate::{Error, ProcessMap, ReadOutputParams};

const POLL_MS: u64 = 50;

pub async fn read_output(
    processes: &ProcessMap,
    params: ReadOutputParams,
) -> Result<CallToolResult, Error> {
    let ReadOutputParams {
        pid,
        timeout_ms,
        max_bytes,
    } = params;

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    let (text, alive) = loop {
        {
            let map = processes.lock().await;
            let handle = map.get(&pid).ok_or(Error::UnknownPid(pid))?;
            let alive = handle.alive.load(Ordering::Relaxed);
            let mut buf = handle.output.lock().await;
            if buf.cursor < buf.end() || !alive || std::time::Instant::now() >= deadline {
                break (buf.take_new(), alive);
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(POLL_MS)).await;
    };

    let text = tail(&text, max_bytes);

    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "alive={alive}\n{text}"
    ))]))
}

/// Keep the most recent `max_bytes`, starting at a line boundary, noting what was dropped.
// needed helper:
fn tail(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let cut = text.len() - max_bytes;
    let start = match text[cut..].find('\n') {
        Some(i) => cut + i + 1,
        None => cut,
    };
    format!(
        "...dropped {} earlier bytes; raise max_bytes to see them...\n{}",
        start,
        &text[start..]
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage_tail() {
        assert_eq!(super::tail("short\n", 100), "short\n");
        // Keeps the most recent whole lines that fit; a line straddling the cut is dropped.
        let long = "aaaa\nbbbb\ncccc\ndddd\n";
        let out = super::tail(long, 10);
        assert!(out.contains("dropped 15 earlier bytes"));
        assert!(out.ends_with("dddd\n"));
        assert!(!out.contains("aaaa"));
    }
}

#[cfg(test)]
mod integration_tests {
    use std::{
        collections::HashMap,
        sync::{Arc, atomic::AtomicU32},
    };

    use tokio::sync::Mutex;

    use super::read_output;
    use crate::{ProcessMap, ReadOutputParams, SpawnParams, server_method};

    #[tokio::test]
    async fn test_usage() {
        let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU32::new(1));
        let params = SpawnParams {
            cmd: "echo".to_string(),
            args: vec!["hello-read".to_string()],
            cwd: None,
            env: HashMap::new(),
            send_to_telegram: true,
        };
        server_method::spawn_process(&processes, &next_id, params)
            .await
            .ok();
        let result = read_output(
            &processes,
            ReadOutputParams {
                pid: 1,
                timeout_ms: 500,
                max_bytes: 32 * 1024,
            },
        )
        .await;
        assert!(result.is_ok());
        let text = format!("{:?}", result.ok());
        assert!(text.contains("hello-read"));
        crate::server_method::kill_process(&processes, 1).await.ok();
    }

    #[tokio::test]
    async fn test_returns_early_on_new_output() {
        let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU32::new(1));
        let params = SpawnParams {
            cmd: "sh".to_string(),
            args: vec!["-c".to_string(), "echo early-line; sleep 30".to_string()],
            cwd: None,
            env: HashMap::new(),
            send_to_telegram: true,
        };
        server_method::spawn_process(&processes, &next_id, params)
            .await
            .ok();
        let start = std::time::Instant::now();
        let result = read_output(
            &processes,
            ReadOutputParams {
                pid: 1,
                timeout_ms: 20_000,
                max_bytes: 32 * 1024,
            },
        )
        .await;
        let elapsed = start.elapsed();
        assert!(result.is_ok());
        let text = format!("{:?}", result.ok());
        assert!(text.contains("early-line"));
        assert!(elapsed < std::time::Duration::from_secs(5));
        crate::server_method::kill_process(&processes, 1).await.ok();
    }
}
