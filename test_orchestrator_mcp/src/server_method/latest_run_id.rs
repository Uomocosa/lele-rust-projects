use serde_json::Value;

use crate::Error;
use crate::server_method;

pub async fn latest_run_id(repo: &str, token: Option<&str>) -> Result<u64, Error> {
    let args = [
        "run".to_string(),
        "list".to_string(),
        "-R".to_string(),
        repo.to_string(),
        "--limit".to_string(),
        "1".to_string(),
        "--json".to_string(),
        "databaseId".to_string(),
    ];
    let out = server_method::run_gh(token, &args).await?;
    let value: Value =
        serde_json::from_str(&out).map_err(|e| Error::BadJson("runs", e.to_string()))?;
    let runs = value.as_array().ok_or(Error::EmptyResponse("runs"))?;
    let first = runs.first().ok_or(Error::EmptyResponse("runs"))?;
    let id = first
        .get("databaseId")
        .and_then(Value::as_u64)
        .ok_or(Error::EmptyResponse("runs"))?;
    Ok(id)
}
// no test_usage necessary — requires live gh CLI + network; exercised by the MCP smoke test
