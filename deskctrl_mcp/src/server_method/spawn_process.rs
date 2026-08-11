use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

use rmcp::model::{CallToolResult, ContentBlock};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::{Mutex, oneshot},
};

use crate::{Error, OutputBuffer, ProcessHandle, ProcessMap, SpawnParams};

pub async fn spawn_process(
    processes: &ProcessMap,
    next_id: &Arc<AtomicU32>,
    params: SpawnParams,
) -> Result<CallToolResult, Error> {
    let SpawnParams {
        cmd,
        args,
        cwd,
        env,
        ..
    } = params;

    let mut builder = Command::new(&cmd);
    builder.args(&args);
    if let Some(ref dir) = cwd {
        builder.current_dir(dir);
    }
    if !env.is_empty() {
        builder.envs(&env);
    }
    builder
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped());

    let mut child = builder.spawn().map_err(Error::Spawn)?;

    // Must be read before `child` is moved into the waiter task below.
    let os_pid = child.id();

    let stdout = child.stdout.take().ok_or(Error::PipeUnavailable)?;
    let stderr = child.stderr.take().ok_or(Error::PipeUnavailable)?;
    let stdin = child.stdin.take().ok_or(Error::PipeUnavailable)?;

    let output: Arc<Mutex<OutputBuffer>> = Arc::new(Mutex::new(OutputBuffer::default()));
    let alive = Arc::new(AtomicBool::new(true));

    let buf = output.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            buf.lock().await.push(&format!("[OUT] {line}\n"));
        }
    });

    let buf = output.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            buf.lock().await.push(&format!("[ERR] {line}\n"));
        }
    });

    let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<String>(32);
    tokio::spawn(async move {
        let mut writer = stdin;
        while let Some(text) = stdin_rx.recv().await {
            if writer.write_all(text.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    let (kill_tx, kill_rx) = oneshot::channel::<()>();
    let alive_wait = alive.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = child.wait() => {}
            _ = kill_rx => { let _ = child.kill().await; }
        }
        alive_wait.store(false, Ordering::Relaxed);
    });

    let id = next_id.fetch_add(1, Ordering::Relaxed);
    let handle = ProcessHandle {
        cmd: format!("{cmd} {}", args.join(" ")),
        os_pid,
        output,
        stdin_tx: Some(stdin_tx),
        alive,
        kill_tx: Some(kill_tx),
    };
    processes.lock().await.insert(id, handle);

    let os_pid_text = match os_pid {
        Some(p) => format!(" (os_pid {p})"),
        None => String::new(),
    };
    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "spawned process {id}{os_pid_text}"
    ))]))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, atomic::AtomicU32},
    };

    use tokio::sync::Mutex;

    use super::spawn_process;
    use crate::{ProcessMap, SpawnParams};

    #[tokio::test]
    async fn test_usage() {
        let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU32::new(1));
        let params = SpawnParams {
            cmd: "echo".to_string(),
            args: vec!["hello-mcp".to_string()],
            cwd: None,
            env: HashMap::new(),
            send_to_telegram: true,
        };
        let result = spawn_process(&processes, &next_id, params).await;
        assert!(result.is_ok());
        assert!(processes.lock().await.contains_key(&1));
        crate::server_method::kill_process(&processes, 1).await.ok();
    }
}
