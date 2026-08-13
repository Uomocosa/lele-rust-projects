use rmcp::model::{CallToolResult, ContentBlock};

use crate::Error;
use crate::StopGameParams;

pub async fn stop_game(params: StopGameParams) -> Result<CallToolResult, Error> {
    let pid = *params;
    if pid < 2 || pid > i32::MAX as u32 {
        return Err(Error::InvalidPid(pid, i32::MAX as u32));
    }
    let out = kill_command(pid).output().map_err(Error::Io)?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(Error::KillFailed(pid, stderr));
    }
    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "terminated process {pid}"
    ))]))
}

// needed helper:
fn kill_command(pid: u32) -> std::process::Command {
    #[cfg(unix)]
    {
        let mut cmd = std::process::Command::new("kill");
        cmd.arg(pid.to_string());
        cmd
    }
    #[cfg(windows)]
    {
        let mut cmd = std::process::Command::new("taskkill");
        cmd.args(["/PID", &pid.to_string(), "/T"]);
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::stop_game;
    use crate::StopGameParams;

    #[tokio::test]
    async fn test_usage() {
        for bad in [0u32, 1, u32::MAX, u32::MAX - 1] {
            let err = stop_game(StopGameParams(bad)).await.unwrap_err();
            assert!(err.to_string().contains("refusing to kill"), "pid {bad}");
        }
        let err = stop_game(StopGameParams(2_147_483_000)).await.unwrap_err();
        assert!(err.to_string().contains("failed to kill process"));
    }
}
