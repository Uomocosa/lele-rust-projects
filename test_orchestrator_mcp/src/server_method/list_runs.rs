use rmcp::model::{CallToolResult, ContentBlock};
use serde_json::Value;

use crate::Error;
use crate::server_method;

pub async fn list_runs(repo: &str, token: Option<&str>) -> Result<CallToolResult, Error> {
    let args = [
        "run".to_string(),
        "list".to_string(),
        "-R".to_string(),
        repo.to_string(),
        "--limit".to_string(),
        "10".to_string(),
        "--json".to_string(),
        "databaseId,workflowName,status,conclusion,displayTitle".to_string(),
    ];
    let out = server_method::run_gh(token, &args).await?;
    let value: Value =
        serde_json::from_str(&out).map_err(|e| Error::BadJson("runs", e.to_string()))?;
    let runs = value.as_array().ok_or(Error::EmptyResponse("runs"))?;
    if runs.is_empty() {
        return Ok(CallToolResult::success(vec![ContentBlock::text(
            "no workflow runs yet",
        )]));
    }
    let lines: Vec<String> = runs
        .iter()
        .map(|r| {
            let id = r.get("databaseId").and_then(Value::as_u64).unwrap_or(0);
            let workflow = r.get("workflowName").and_then(Value::as_str).unwrap_or("?");
            let status = r.get("status").and_then(Value::as_str).unwrap_or("?");
            let conclusion = r.get("conclusion").and_then(Value::as_str).unwrap_or("-");
            let title = r.get("displayTitle").and_then(Value::as_str).unwrap_or("?");
            format!("#{id} {workflow}: {status}/{conclusion} — {title}")
        })
        .collect();
    Ok(CallToolResult::success(vec![ContentBlock::text(
        lines.join("\n"),
    )]))
}
// no test_usage necessary — requires live gh CLI + network; exercised by the MCP smoke test
