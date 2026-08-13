use rmcp::model::{CallToolResult, ContentBlock};

use crate::Error;
use crate::GameStatusParams;

const KEYWORDS: [&str; 8] = [
    "ring_connections",
    "roster",
    "ConnectionEstablished",
    "ConnectionClosed",
    "retrying embedded node",
    "EADDRINUSE",
    "error",
    "wait_ready",
];

pub async fn game_status(params: GameStatusParams) -> Result<CallToolResult, Error> {
    let log_file = params.log_file.as_deref();
    let pid = params.pid;
    let log = log_file.unwrap_or("fbx_game.log");
    let content =
        std::fs::read_to_string(log).map_err(|e| Error::LogRead(log.to_string(), e.to_string()))?;
    let mut counts = std::collections::BTreeMap::new();
    let mut matches: Vec<&str> = Vec::new();
    for line in content.lines() {
        if let Some(keyword) = KEYWORDS.iter().find(|k| line.contains(*k)) {
            *counts.entry(*keyword).or_insert(0usize) += 1;
            matches.push(line.trim());
        }
    }
    let mut lines = Vec::new();
    if let Some(pid) = pid {
        let alive = pid_alive(pid);
        lines.push(format!(
            "process {pid}: {}",
            if alive { "alive" } else { "dead" }
        ));
    }
    lines.push(format!(
        "log {}: {} lines, {} events",
        log,
        content.lines().count(),
        matches.len()
    ));
    for (keyword, count) in &counts {
        lines.push(format!("{keyword}: {count}"));
    }
    let tail: Vec<&str> = matches.iter().rev().take(30).rev().copied().collect();
    if !tail.is_empty() {
        lines.push("--- last matching lines ---".to_string());
        lines.extend(tail.iter().map(|l| format!("  {l}")));
    }
    Ok(CallToolResult::success(vec![ContentBlock::text(
        lines.join("\n"),
    )]))
}

// needed helper:
fn pid_alive(pid: u32) -> bool {
    if pid < 2 || pid > i32::MAX as u32 {
        return false;
    }
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}")])
            .output()
            .map(|o| {
                let out = String::from_utf8_lossy(&o.stdout);
                out.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::{game_status, pid_alive};
    use crate::GameStatusParams;

    #[tokio::test]
    async fn test_usage() {
        let params = GameStatusParams {
            log_file: Some("/nonexistent/fbx.log".into()),
            pid: None,
        };
        let err = game_status(params).await.unwrap_err();
        assert!(err.to_string().contains("failed to read log"));
        assert!(!pid_alive(u32::MAX));
        assert!(!pid_alive(0));
        assert!(!pid_alive(2_147_483_000));
    }
}
