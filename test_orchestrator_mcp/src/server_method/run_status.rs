use rmcp::model::{CallToolResult, ContentBlock};
use serde_json::Value;

use crate::Error;
use crate::RunStatusParams;
use crate::server_method::{latest_run_id, run_gh};

pub async fn run_status(
    repo: &str,
    token: Option<&str>,
    params: RunStatusParams,
) -> Result<CallToolResult, Error> {
    let run_id = match *params {
        Some(id) => id,
        None => latest_run_id(repo, token).await?,
    };
    let args = [
        "run".to_string(),
        "view".to_string(),
        run_id.to_string(),
        "-R".to_string(),
        repo.to_string(),
        "--json".to_string(),
        "status,conclusion,displayTitle,url,jobs".to_string(),
    ];
    let out = run_gh(token, &args).await?;
    let value: Value =
        serde_json::from_str(&out).map_err(|e| Error::BadJson("run", e.to_string()))?;
    let status = value.get("status").and_then(Value::as_str).unwrap_or("?");
    let conclusion = value
        .get("conclusion")
        .and_then(Value::as_str)
        .unwrap_or("-");
    let title = value
        .get("displayTitle")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let url = value.get("url").and_then(Value::as_str).unwrap_or("?");
    let mut lines = vec![format!("#{run_id} {title}: {status}/{conclusion}\n{url}")];
    if let Some(jobs) = value.get("jobs").and_then(Value::as_array) {
        for job in jobs {
            let name = job.get("name").and_then(Value::as_str).unwrap_or("?");
            let job_status = job.get("status").and_then(Value::as_str).unwrap_or("?");
            let job_conclusion = job.get("conclusion").and_then(Value::as_str).unwrap_or("-");
            lines.push(format!("  {name}: {job_status}/{job_conclusion}"));
        }
    }
    Ok(CallToolResult::success(vec![ContentBlock::text(
        lines.join("\n"),
    )]))
}
// no test_usage necessary — requires live gh CLI + network; exercised by the MCP smoke test
