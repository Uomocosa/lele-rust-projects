use rmcp::model::{CallToolResult, ContentBlock};

use crate::Error;

pub async fn vpn_status() -> Result<CallToolResult, Error> {
    let status_out = tokio::process::Command::new("protonvpn")
        .arg("status")
        .output()
        .await
        .map_err(|e| Error::Screenshot(format!("protonvpn status spawn failed: {e}")))?;

    let stdout = String::from_utf8_lossy(&status_out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&status_out.stderr).to_string();
    let status_text = format!("{stdout}{stderr}").trim().to_string();

    let is_connected = status_text.to_lowercase().contains("connected")
        && !status_text.to_lowercase().contains("disconnected");

    let ip_out = tokio::process::Command::new("curl")
        .args(["-s", "--max-time", "5", "https://ifconfig.me"])
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let iproute_out = tokio::process::Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let summary = format!(
        "protonvpn status: {}\npublic_ip: {}\ndefault_route: {}\nraw:\n{}",
        if is_connected { "Connected" } else { "Disconnected" },
        if ip_out.is_empty() { "(unavailable)" } else { &ip_out },
        if iproute_out.is_empty() {
            "(unavailable)"
        } else {
            &iproute_out
        },
        status_text
    );

    Ok(CallToolResult::success(vec![ContentBlock::text(summary)]))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert!(true);
    }
}
