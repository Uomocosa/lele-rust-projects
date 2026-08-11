use rmcp::model::{CallToolResult, ContentBlock};

use crate::{Error, ProcessMap};

pub async fn kill_process(processes: &ProcessMap, pid: u32) -> Result<CallToolResult, Error> {
    let mut map = processes.lock().await;
    let handle = map.remove(&pid).ok_or(Error::UnknownPid(pid))?;

    if let Some(tx) = handle.kill_tx {
        let _ = tx.send(());
    }

    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "killed process {pid}"
    ))]))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, atomic::AtomicU32},
    };

    use tokio::sync::Mutex;

    use super::kill_process;
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
        let result = kill_process(&processes, 1).await;
        assert!(result.is_ok());
        assert!(processes.lock().await.is_empty());
    }
}
