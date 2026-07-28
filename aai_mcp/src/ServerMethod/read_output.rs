use std::sync::atomic::Ordering;

use rmcp::model::{CallToolResult, ContentBlock};

use crate::{Error, ProcessMap, ReadOutputParams};

const POLL_MS: u64 = 50;

pub async fn read_output(
    processes: &ProcessMap,
    params: ReadOutputParams,
) -> Result<CallToolResult, Error> {
    let ReadOutputParams { pid, timeout_ms } = params;

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    let (text, alive) = loop {
        {
            let map = processes.lock().await;
            let handle = map.get(&pid).ok_or(Error::UnknownPid(pid))?;
            let alive = handle.alive.load(Ordering::Relaxed);
            let mut buf = handle.output_buf.lock().await;
            if !buf.is_empty() || !alive || std::time::Instant::now() >= deadline {
                break (buf.drain(..).collect::<String>(), alive);
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(POLL_MS)).await;
    };

    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "alive={alive}\n{text}"
    ))]))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, atomic::AtomicU32},
    };

    use tokio::sync::Mutex;

    use super::read_output;
    use crate::{ProcessMap, ReadOutputParams, ServerMethod, SpawnParams};

    #[tokio::test]
    async fn test_usage() {
        let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU32::new(1));
        let params = SpawnParams {
            cmd: "echo".to_string(),
            args: vec!["hello-read".to_string()],
            cwd: None,
            env: HashMap::new(),
        };
        ServerMethod::spawn_process(&processes, &next_id, params)
            .await
            .ok();
        let result = read_output(
            &processes,
            ReadOutputParams {
                pid: 1,
                timeout_ms: 500,
            },
        )
        .await;
        assert!(result.is_ok());
        let text = format!("{:?}", result.ok());
        assert!(text.contains("hello-read"));
        crate::ServerMethod::kill_process(&processes, 1).await.ok();
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
        };
        ServerMethod::spawn_process(&processes, &next_id, params)
            .await
            .ok();
        let start = std::time::Instant::now();
        let result = read_output(
            &processes,
            ReadOutputParams {
                pid: 1,
                timeout_ms: 20_000,
            },
        )
        .await;
        let elapsed = start.elapsed();
        assert!(result.is_ok());
        let text = format!("{:?}", result.ok());
        assert!(text.contains("early-line"));
        assert!(elapsed < std::time::Duration::from_secs(5));
        crate::ServerMethod::kill_process(&processes, 1).await.ok();
    }
}
