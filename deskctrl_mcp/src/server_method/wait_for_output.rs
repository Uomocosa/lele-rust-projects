use std::sync::atomic::Ordering;

use rmcp::model::{CallToolResult, ContentBlock};

use crate::{Error, ProcessMap, WaitForOutputParams};

const POLL_MS: u64 = 100;
/// Upper bound on a single call, kept well under any MCP client's per-tool timeout.
/// Waiting longer is done by calling again — the transcript is append-only, so nothing is missed.
pub const MAX_TIMEOUT_MS: u64 = 120_000;

pub async fn wait_for_output(
    processes: &ProcessMap,
    params: WaitForOutputParams,
) -> Result<CallToolResult, Error> {
    let WaitForOutputParams {
        pid,
        pattern,
        timeout_ms,
    } = params;
    let timeout_ms = timeout_ms.min(MAX_TIMEOUT_MS);

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    let (found, alive) = loop {
        {
            let map = processes.lock().await;
            let handle = map.get(&pid).ok_or(Error::UnknownPid(pid))?;
            let alive = handle.alive.load(Ordering::Relaxed);
            // Scans the whole transcript, so a line already returned by read_output still counts.
            let found = handle.output.lock().await.find_line(&pattern);
            if found.is_some() || !alive || std::time::Instant::now() >= deadline {
                break (found, alive);
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(POLL_MS)).await;
    };

    let text = match found {
        Some(line) => format!("matched=true alive={alive}\n{line}"),
        None if !alive => {
            format!("matched=false alive=false\nprocess exited before {pattern:?} appeared")
        }
        None => format!(
            "matched=false alive=true\ntimed out after {timeout_ms}ms; call again to keep waiting"
        ),
    };

    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, atomic::AtomicU32},
    };

    use tokio::sync::Mutex;

    use super::wait_for_output;
    use crate::{ProcessMap, ReadOutputParams, SpawnParams, WaitForOutputParams, server_method};

    #[tokio::test]
    async fn test_usage() {
        let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU32::new(1));
        let params = SpawnParams {
            cmd: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "echo warming up; sleep 0.3; echo connected, running key=abc".to_string(),
            ],
            cwd: None,
            env: HashMap::new(),
            send_to_telegram: true,
        };
        server_method::spawn_process(&processes, &next_id, params)
            .await
            .ok();

        let result = wait_for_output(
            &processes,
            WaitForOutputParams {
                pid: 1,
                pattern: "connected, running".to_string(),
                timeout_ms: 10_000,
            },
        )
        .await
        .unwrap();
        let text = format!("{result:?}");
        assert!(text.contains("matched=true"));
        assert!(text.contains("key=abc"));

        crate::server_method::kill_process(&processes, 1).await.ok();
    }

    #[tokio::test]
    async fn test_finds_output_already_drained_by_read_output() {
        let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU32::new(1));
        let params = SpawnParams {
            cmd: "sh".to_string(),
            args: vec!["-c".to_string(), "echo the-marker; sleep 30".to_string()],
            cwd: None,
            env: HashMap::new(),
            send_to_telegram: true,
        };
        server_method::spawn_process(&processes, &next_id, params)
            .await
            .ok();

        // read_output consumes the line first...
        let read = server_method::read_output(
            &processes,
            ReadOutputParams {
                pid: 1,
                timeout_ms: 5_000,
                max_bytes: 32 * 1024,
            },
        )
        .await
        .unwrap();
        assert!(format!("{read:?}").contains("the-marker"));

        // ...and wait_for_output must still find it.
        let result = wait_for_output(
            &processes,
            WaitForOutputParams {
                pid: 1,
                pattern: "the-marker".to_string(),
                timeout_ms: 1_000,
            },
        )
        .await
        .unwrap();
        assert!(format!("{result:?}").contains("matched=true"));

        crate::server_method::kill_process(&processes, 1).await.ok();
    }

    #[tokio::test]
    async fn test_reports_timeout_without_blocking_forever() {
        let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU32::new(1));
        let params = SpawnParams {
            cmd: "sh".to_string(),
            args: vec!["-c".to_string(), "sleep 30".to_string()],
            cwd: None,
            env: HashMap::new(),
            send_to_telegram: true,
        };
        server_method::spawn_process(&processes, &next_id, params)
            .await
            .ok();

        let result = wait_for_output(
            &processes,
            WaitForOutputParams {
                pid: 1,
                pattern: "never-appears".to_string(),
                timeout_ms: 300,
            },
        )
        .await
        .unwrap();
        assert!(format!("{result:?}").contains("matched=false"));

        crate::server_method::kill_process(&processes, 1).await.ok();
    }
}
