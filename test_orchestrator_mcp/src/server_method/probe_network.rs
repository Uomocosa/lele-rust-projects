use std::time::Duration;

use rmcp::model::{CallToolResult, ContentBlock};

use crate::Error;
use crate::server_method::{latest_workflow_run_id, run_gh};

pub async fn probe_network(repo: &str, token: Option<&str>) -> Result<CallToolResult, Error> {
    run_gh(
        token,
        &[
            "workflow".to_string(),
            "run".to_string(),
            "network-probe.yml".to_string(),
            "-R".to_string(),
            repo.to_string(),
        ],
    )
    .await?;

    let run_id = latest_workflow_run_id(repo, "network-probe.yml", token).await?;

    let args = [
        "run".to_string(),
        "watch".to_string(),
        run_id.to_string(),
        "-R".to_string(),
        repo.to_string(),
        "--exit-status".to_string(),
        "--interval".to_string(),
        "10".to_string(),
    ];
    let watch = run_gh(token, &args);

    match tokio::time::timeout(Duration::from_secs(180), watch).await {
        Err(_) => {
            return Err(Error::GhFailed(format!(
                "network probe run #{run_id} timed out after 180s — a self-hosted runner is offline. Windows: run C:\\actions-runner\\run.cmd; Linux: ~/actions-runner/run.sh"
            )));
        }
        Ok(Err(e)) => {
            return Err(match e {
                Error::GhFailed(msg) => {
                    Error::GhFailed(format!("network probe run #{run_id} failed: {msg}"))
                }
                other => other,
            });
        }
        Ok(Ok(_)) => {}
    }

    let dest = std::env::temp_dir().join(format!("network-probe-{run_id}"));
    run_gh(
        token,
        &[
            "run".to_string(),
            "download".to_string(),
            run_id.to_string(),
            "-R".to_string(),
            repo.to_string(),
            "-D".to_string(),
            dest.to_string_lossy().to_string(),
        ],
    )
    .await?;

    let mut lines = Vec::new();
    for file in read_json_files(&dest) {
        match std::fs::read_to_string(&file) {
            Ok(content) => lines.push(content.trim().to_string()),
            Err(e) => lines.push(format!("{}: {e}", file.display())),
        }
    }
    if lines.is_empty() {
        return Err(Error::EmptyResponse("network-probe artifact"));
    }
    Ok(CallToolResult::success(vec![ContentBlock::text(
        lines.join("\n"),
    )]))
}

// needed helper:
fn read_json_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "json") {
                out.push(path);
            } else if path.is_dir() {
                out.extend(read_json_files(&path));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::read_json_files;

    #[test]
    fn test_usage() {
        assert!(read_json_files(std::path::Path::new("/nonexistent")).is_empty());
    }
}
