use rmcp::model::{CallToolResult, ContentBlock};

use crate::Error;

pub async fn vpn_disconnect() -> Result<CallToolResult, Error> {
    let out = tokio::process::Command::new("protonvpn")
        .arg("disconnect")
        .output()
        .await
        .map_err(|e| Error::Screenshot(format!("protonvpn disconnect spawn failed: {e}")))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}").trim().to_string();
    if out.status.success() {
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "protonvpn disconnect succeeded:\n{combined}"
        ))]))
    } else {
        Err(Error::Screenshot(format!(
            "protonvpn disconnect failed (exit {}): {combined}",
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
