use serde_json::Value;

use crate::Error;
use crate::NextTagParams;
use crate::server_method::{run_gh, validate_mode};

pub async fn next_tag(
    repo: &str,
    token: Option<&str>,
    params: NextTagParams,
) -> Result<String, Error> {
    validate_mode(&params.mode)?;
    let crate_folder = params
        .crate_folder
        .unwrap_or_else(|| "freenet_libp2p_bevy_example_1".to_string());
    let normalized = normalize_crate(&crate_folder);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let prefix = format!("{normalized}-{}-{today}#", params.mode);
    let args = ["api".to_string(), format!("repos/{repo}/tags?per_page=100")];
    let out = run_gh(token, &args).await?;
    let value: Value =
        serde_json::from_str(&out).map_err(|e| Error::BadJson("tags", e.to_string()))?;
    let tags = value.as_array().ok_or(Error::EmptyResponse("tags"))?;
    let max = tags
        .iter()
        .filter_map(|t| t.get("name").and_then(Value::as_str))
        .filter_map(|name| name.strip_prefix(&prefix))
        .filter_map(|suffix| suffix.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    Ok(format!("{prefix}{}", max + 1))
}

// needed helper:
fn normalize_crate(folder: &str) -> String {
    folder.replace(['_', ' '], "-")
}

#[cfg(test)]
mod tests {
    use super::next_tag;
    use crate::NextTagParams;

    #[tokio::test]
    async fn test_usage() {
        let params = NextTagParams {
            mode: "nope".into(),
            crate_folder: None,
        };
        let err = next_tag("repo", None, params).await.unwrap_err();
        assert!(err.to_string().contains("invalid mode"));
    }
}
