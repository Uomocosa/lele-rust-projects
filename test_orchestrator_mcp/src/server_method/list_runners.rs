use rmcp::model::{CallToolResult, ContentBlock};
use serde_json::Value;

use crate::Error;
use crate::server_method;

pub async fn list_runners(repo: &str, token: Option<&str>) -> Result<CallToolResult, Error> {
    let args = [
        "api".to_string(),
        format!("repos/{repo}/actions/runners"),
        "--paginate".to_string(),
    ];
    let out = server_method::run_gh(token, &args).await?;
    let value: Value =
        serde_json::from_str(&out).map_err(|e| Error::BadJson("runners", e.to_string()))?;
    let runners = value
        .get("runners")
        .and_then(Value::as_array)
        .ok_or(Error::EmptyResponse("runners"))?;
    if runners.is_empty() {
        return Ok(CallToolResult::success(vec![ContentBlock::text(
            "no self-hosted runners registered",
        )]));
    }
    let mut lines: Vec<String> = runners
        .iter()
        .map(|r| {
            let name = r.get("name").and_then(Value::as_str).unwrap_or("?");
            let status = r.get("status").and_then(Value::as_str).unwrap_or("?");
            let labels = r
                .get("labels")
                .and_then(Value::as_array)
                .map(|l| {
                    l.iter()
                        .filter_map(|x| x.get("name").and_then(Value::as_str))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .unwrap_or_default();
            format!("{name}: {status} [{labels}]")
        })
        .collect();
    lines.sort();
    Ok(CallToolResult::success(vec![ContentBlock::text(
        lines.join("\n"),
    )]))
}
// no test_usage necessary — requires live gh CLI + network; exercised by the MCP smoke test
