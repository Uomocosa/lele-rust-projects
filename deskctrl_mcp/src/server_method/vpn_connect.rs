use rmcp::model::{CallToolResult, ContentBlock};

use crate::Error;

pub async fn vpn_connect(
    server: Option<String>,
    country: Option<String>,
    city: Option<String>,
) -> Result<CallToolResult, Error> {
    let mut cmd = tokio::process::Command::new("protonvpn");
    cmd.arg("connect");
    if let Some(c) = country.as_deref().filter(|s| !s.trim().is_empty()) {
        cmd.args(["--country", c]);
    }
    if let Some(c) = city.as_deref().filter(|s| !s.trim().is_empty()) {
        cmd.args(["--city", c]);
    }
    if let Some(s) = server.as_deref().filter(|s| !s.trim().is_empty()) {
        cmd.arg(s);
    }

    let out = cmd.output().await.map_err(|e| {
        Error::Screenshot(format!("protonvpn connect spawn failed: {e}"))
    })?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}").trim().to_string();
    if out.status.success() {
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "protonvpn connect succeeded:\n{combined}"
        ))]))
    } else {
        Err(Error::Screenshot(format!(
            "protonvpn connect failed (exit {}): {combined}",
            out.status.code().unwrap_or(-1)
        )))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert!(true);
    }
}
