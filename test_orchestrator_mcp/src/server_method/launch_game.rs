use rmcp::model::{CallToolResult, ContentBlock};
use std::path::Path;
use std::process::Stdio;

use crate::Error;
use crate::LaunchGameParams;

pub async fn launch_game(
    exe_override: Option<&str>,
    params: LaunchGameParams,
) -> Result<CallToolResult, Error> {
    let exe = default_exe(exe_override);
    if !Path::new(&exe).exists() {
        return Err(Error::ExeNotFound(exe));
    }
    let log = params
        .log_file
        .unwrap_or_else(|| "fbx_game.log".to_string());
    let log_out =
        std::fs::File::create(&log).map_err(|e| Error::LogCreate(log.clone(), e.to_string()))?;
    let log_err = log_out
        .try_clone()
        .map_err(|e| Error::LogCreate(log.clone(), e.to_string()))?;
    let mut cmd = tokio::process::Command::new(&exe);
    cmd.arg("--identity-dir")
        .arg(&params.identity_dir)
        .arg("--p2p-port")
        .arg(params.p2p_port.to_string())
        .env("RUST_LOG", "warn,roster=info,p2p=info")
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_out))
        .stderr(Stdio::from(log_err));
    let child = cmd.spawn().map_err(Error::Spawn)?;
    let pid = child.id().ok_or(Error::NoPid)?;
    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "launched {exe} (pid {pid}), identity {}, p2p port {}, log {log}",
        params.identity_dir, params.p2p_port
    ))]))
}

// needed helper:
fn default_exe(exe_override: Option<&str>) -> String {
    if let Some(exe) = exe_override {
        return exe.to_string();
    }
    let bin = if cfg!(windows) {
        "freenet-libp2p-bevy-example-1.exe"
    } else {
        "freenet-libp2p-bevy-example-1"
    };
    let base = format!(
        "{}/../freenet_libp2p_bevy_example_1/target",
        env!("CARGO_MANIFEST_DIR")
    );
    for sub in ["ci/release", "ci/debug", "release", "debug"] {
        let candidate = format!("{base}/{sub}/{bin}");
        if Path::new(&candidate).exists() {
            return candidate;
        }
    }
    format!("{base}/ci/release/{bin}")
}

#[cfg(test)]
mod tests {
    use super::{default_exe, launch_game};
    use crate::LaunchGameParams;

    #[tokio::test]
    async fn test_usage() {
        let params = LaunchGameParams {
            exe: Some("/nonexistent/fbx".into()),
            identity_dir: "/tmp/fbx_test".into(),
            p2p_port: 63221,
            log_file: None,
        };
        let err = launch_game(Some("/nonexistent/fbx"), params)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("game executable not found"));
        assert!(default_exe(Some("/x")) == "/x");
    }
}
