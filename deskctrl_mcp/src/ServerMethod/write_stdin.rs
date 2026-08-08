use rmcp::model::{CallToolResult, ContentBlock};

use crate::{Error, ProcessMap, WriteStdinParams};

pub async fn write_stdin(
    processes: &ProcessMap,
    params: WriteStdinParams,
) -> Result<CallToolResult, Error> {
    let WriteStdinParams { pid, text } = params;
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
    use crate::{ProcessMap, ReadOutputParams, ServerMethod, SpawnParams, WriteStdinParams};

    #[tokio::test]
    async fn test_usage() {
        let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU32::new(1));
        let params = SpawnParams {
            cmd: "cat".to_string(),
            args: vec![],
            cwd: None,
            env: HashMap::new(),
        };
        ServerMethod::spawn_process(&processes, &next_id, params)
            .await
            .ok();
        let result = write_stdin(
            &processes,
            WriteStdinParams {
                pid: 1,
                text: "ping-stdin".to_string(),
            },
        )
        .await;
        assert!(result.is_ok());
        let echoed = ServerMethod::read_output(
            &processes,
            ReadOutputParams {
                pid: 1,
                timeout_ms: 500,
            },
        )
        .await;
        let text = format!("{:?}", echoed.ok());
        assert!(text.contains("ping-stdin"));
        crate::ServerMethod::kill_process(&processes, 1).await.ok();
    }
}
