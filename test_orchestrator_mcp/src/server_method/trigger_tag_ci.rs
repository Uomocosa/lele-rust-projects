use rmcp::model::{CallToolResult, ContentBlock};
use serde_json::Value;

use crate::Error;
use crate::NextTagParams;
use crate::TriggerTagCiParams;
use crate::server_method::{next_tag, run_gh, validate_mode};

pub async fn trigger_tag_ci(
    repo: &str,
    token: Option<&str>,
    params: TriggerTagCiParams,
) -> Result<CallToolResult, Error> {
    validate_mode(&params.mode)?;
    let dry_run = params.dry_run.unwrap_or(false);
    let tag = next_tag(
        repo,
        token,
        NextTagParams {
            mode: params.mode.clone(),
            crate_folder: params.crate_folder.clone(),
        },
    )
    .await?;
    if dry_run {
        return Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "dry run: would push tag {tag}"
        ))]));
    }
    let head_args = ["api".to_string(), format!("repos/{repo}")];
    let head_out = run_gh(token, &head_args).await?;
    let head_value: Value =
        serde_json::from_str(&head_out).map_err(|e| Error::BadJson("repo", e.to_string()))?;
    let branch = head_value
        .get("default_branch")
        .and_then(Value::as_str)
        .ok_or(Error::EmptyResponse("repo"))?;
    let ref_args = [
        "api".to_string(),
        format!("repos/{repo}/git/ref/heads/{branch}"),
    ];
    let ref_out = run_gh(token, &ref_args).await?;
    let ref_value: Value =
        serde_json::from_str(&ref_out).map_err(|e| Error::BadJson("ref", e.to_string()))?;
    let sha = ref_value
        .pointer("/object/sha")
        .and_then(Value::as_str)
        .ok_or(Error::EmptyResponse("ref"))?;
    let push_args = [
        "api".to_string(),
        "-X".to_string(),
        "POST".to_string(),
        format!("repos/{repo}/git/refs"),
        "-f".to_string(),
        format!("ref=refs/tags/{tag}"),
        "-f".to_string(),
        format!("sha={sha}"),
    ];
    run_gh(token, &push_args).await?;
    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "pushed tag {tag} — the crate-tag CI workflow ({} mode) has been triggered on GitHub-hosted runners",
        params.mode
    ))]))
}

#[cfg(test)]
mod tests {
    use super::trigger_tag_ci;
    use crate::TriggerTagCiParams;

    #[tokio::test]
    async fn test_usage() {
        let params = TriggerTagCiParams {
            mode: "bogus".into(),
            crate_folder: None,
            dry_run: None,
        };
        let err = trigger_tag_ci("repo", None, params).await.unwrap_err();
        assert!(err.to_string().contains("invalid mode"));
    }
}
