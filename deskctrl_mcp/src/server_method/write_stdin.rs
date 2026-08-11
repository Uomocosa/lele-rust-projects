use rmcp::model::{CallToolResult, ContentBlock};

use crate::{Error, ProcessMap, WriteStdinParams};

pub async fn write_stdin(
    processes: &ProcessMap,
    params: WriteStdinParams,
) -> Result<CallToolResult, Error> {
    let WriteStdinParams { pid, text, .. } = params;
    let line = if text.ends_with('\n') {
        text
    } else {
        format!("{text}\n")
    };

    let map = processes.lock().await;
    let handle = map.get(&pid).ok_or(Error::UnknownPid(pid))?;

    match &handle.stdin_tx {
        Some(tx) => tx.send(line).await.map_err(|_| Error::StdinClosed)?,
        None => return Err(Error::PipeUnavailable),
    }

    Ok(CallToolResult::success(vec![ContentBlock::text("ok")]))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, atomic::AtomicU32},
    };

    use tokio::sync::Mutex;

    use super::write_stdin;
    use crate::{ProcessMap, ReadOutputParams, SpawnParams, WriteStdinParams, server_method};

    #[tokio::test]
    async fn test_usage() {
        let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU32::new(1));
        let params = SpawnParams {
            cmd: "cat".to_string(),
            args: vec![],
            cwd: None,
            env: HashMap::new(),
            send_to_telegram: true,
        };
        server_method::spawn_process(&processes, &next_id, params)
            .await
            .ok();
        let result = write_stdin(
            &processes,
            WriteStdinParams {
                pid: 1,
                text: "ping-stdin".to_string(),
                send_to_telegram: true,
            },
        )
        .await;
        assert!(result.is_ok());
        let echoed = server_method::read_output(
            &processes,
            ReadOutputParams {
                pid: 1,
                timeout_ms: 500,
                max_bytes: 32 * 1024,
            },
        )
        .await;
        let text = format!("{:?}", echoed.ok());
        assert!(text.contains("ping-stdin"));
        crate::server_method::kill_process(&processes, 1).await.ok();
    }
}
