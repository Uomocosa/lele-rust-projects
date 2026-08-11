use std::sync::atomic::Ordering;

use rmcp::model::{CallToolResult, ContentBlock};
use serde_json::json;

use crate::{Error, ProcessMap};

pub async fn list_processes(processes: &ProcessMap) -> Result<CallToolResult, Error> {
    let map = processes.lock().await;
    let entries: Vec<serde_json::Value> = map
        .iter()
        .map(|(id, handle)| {
            json!({
                "pid": id,
                "os_pid": handle.os_pid,
                "cmd": handle.cmd,
                "alive": handle.alive.load(Ordering::Relaxed),
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![ContentBlock::text(
        serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".into()),
    )]))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, atomic::AtomicU32},
    };

    use tokio::sync::Mutex;

    use super::list_processes;
    use crate::{ProcessMap, SpawnParams, server_method};

    #[tokio::test]
    async fn test_usage() {
        let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU32::new(1));
        let params = SpawnParams {
            cmd: "sleep".to_string(),
            args: vec!["60".to_string()],
            cwd: None,
            env: HashMap::new(),
            send_to_telegram: true,
        };
        server_method::spawn_process(&processes, &next_id, params)
            .await
            .ok();
        let result = list_processes(&processes).await;
        assert!(result.is_ok());
        let text = format!("{:?}", result.ok());
        assert!(text.contains("sleep"));
        crate::server_method::kill_process(&processes, 1).await.ok();
    }
}
