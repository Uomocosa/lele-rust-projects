use std::process::Stdio;

use rmcp::model::{CallToolResult, ContentBlock};
use tokio::io::AsyncWriteExt;

use crate::Error;

fn env_or_file(key: &str) -> Option<String> {
    if let Ok(v) = std::env::var(key)
        && !v.trim().is_empty()
    {
        return Some(v.trim().trim_matches('"').trim_matches('\'').to_string());
    }
    let candidates = [
        concat!(env!("CARGO_MANIFEST_DIR"), "/.env"),
        "../deskctrl_mcp/.env",
        "deskctrl_mcp/.env",
    ];
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=')
                    && k.trim() == key
                {
                    let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                    if !v.is_empty() {
                        return Some(v);
                    }
                }
            }
        }
    }
    None
}

fn proton_creds() -> Option<(String, String)> {
    let user = env_or_file("PROTONVPN_USER")
        .or_else(|| env_or_file("PROTON_VPN_USER"))
        .or_else(|| env_or_file("PROTONVPV_USER"))?;
    let pwd = env_or_file("PROTONVPN_PWD")
        .or_else(|| env_or_file("PROTON_VPN_PWD"))
        .or_else(|| env_or_file("PROTONVPV_PWD"))?;
    Some((user, pwd))
}

async fn run_connect(
    server: Option<&str>,
    country: Option<&str>,
    city: Option<&str>,
) -> Result<(bool, String, i32), Error> {
    let mut cmd = tokio::process::Command::new("protonvpn");
    cmd.arg("connect");
    if let Some(c) = country.filter(|s| !s.trim().is_empty()) {
        cmd.args(["--country", c]);
    }
    if let Some(c) = city.filter(|s| !s.trim().is_empty()) {
        cmd.args(["--city", c]);
    }
    if let Some(s) = server.filter(|s| !s.trim().is_empty()) {
        cmd.arg(s);
    }
    let out = cmd
        .output()
        .await
        .map_err(|e| Error::Screenshot(format!("protonvpn connect spawn failed: {e}")))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}").trim().to_string();
    Ok((out.status.success(), combined, out.status.code().unwrap_or(-1)))
}

async fn try_signin(user: &str, pwd: &str) -> Result<String, Error> {
    let mut child = tokio::process::Command::new("protonvpn")
        .arg("signin")
        .arg(user)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::Screenshot(format!("protonvpn signin spawn failed: {e}")))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(format!("{pwd}\n").as_bytes())
            .await
            .map_err(|e| Error::Screenshot(format!("signin stdin write failed: {e}")))?;
    }
    let out = child
        .wait_with_output()
        .await
        .map_err(|e| Error::Screenshot(format!("signin wait failed: {e}")))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}").trim().to_string();
    if out.status.success() {
        Ok(combined)
    } else {
        Err(Error::Screenshot(format!(
            "protonvpn signin failed (exit {}): {combined}",
            out.status.code().unwrap_or(-1)
        )))
    }
}

pub async fn vpn_connect(
    server: Option<String>,
    country: Option<String>,
    city: Option<String>,
) -> Result<CallToolResult, Error> {
    let (ok, combined, code) =
        run_connect(server.as_deref(), country.as_deref(), city.as_deref()).await?;
    if ok {
        return Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "protonvpn connect succeeded:\n{combined}"
        ))]));
    }
    let needs_auth = combined.to_lowercase().contains("authentication required")
        || combined.to_lowercase().contains("please sign in");
    if needs_auth && let Some((user, pwd)) = proton_creds() {
        let signin_out = try_signin(&user, &pwd).await?;
        let (ok2, combined2, code2) =
            run_connect(server.as_deref(), country.as_deref(), city.as_deref()).await?;
        if ok2 {
            return Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "protonvpn signin succeeded for {user}:\n{signin_out}\n\nprotonvpn connect succeeded:\n{combined2}"
            ))]));
        }
        return Err(Error::Screenshot(format!(
            "protonvpn connect failed after signin (exit {code2}): {combined2} (signin: {signin_out})"
        )));
    }
    Err(Error::Screenshot(format!(
        "protonvpn connect failed (exit {code}): {combined}"
    )))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert!(true);
    }
}
