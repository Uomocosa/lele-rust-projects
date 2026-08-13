use crate::Error;

pub async fn run_gh(token: Option<&str>, args: &[String]) -> Result<String, Error> {
    let mut cmd = tokio::process::Command::new("gh");
    cmd.args(args);
    if let Some(token) = token {
        cmd.env("GH_TOKEN", token);
    }
    let out = cmd.output().await.map_err(Error::GhSpawn)?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(Error::GhFailed(stderr));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::run_gh;

    #[tokio::test]
    async fn test_usage() {
        let out = run_gh(None, &["--version".to_string()]).await.unwrap();
        assert!(out.contains("gh version"));
    }
}
