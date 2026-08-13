use rmcp::model::{CallToolResult, ContentBlock};

use crate::DownloadArtifactsParams;
use crate::Error;
use crate::server_method::{latest_run_id, run_gh};

pub async fn download_artifacts(
    repo: &str,
    token: Option<&str>,
    params: DownloadArtifactsParams,
) -> Result<CallToolResult, Error> {
    let run_id = match params.run_id {
        Some(id) => id,
        None => latest_run_id(repo, token).await?,
    };
    let dest = params.dest.unwrap_or_else(|| "downloads".to_string());
    let mut args = vec![
        "run".to_string(),
        "download".to_string(),
        run_id.to_string(),
        "-R".to_string(),
        repo.to_string(),
        "-D".to_string(),
        dest.clone(),
    ];
    if let Some(pattern) = params.pattern {
        args.push("-n".to_string());
        args.push(pattern);
    }
    run_gh(token, &args).await?;
    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "downloaded artifacts of run #{run_id} to {dest}"
    ))]))
}
// no test_usage necessary — requires live gh CLI + network; exercised by the MCP smoke test
