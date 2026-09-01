use std::path::{Path, PathBuf};
use std::process::Command;

/// # Errors
/// Returns an error if the cargo build or metadata resolution fails.
pub fn build_game() -> Result<PathBuf, String> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let game_dir = manifest.to_path_buf();
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&game_dir)
        .env(
            "CARGO_TARGET_DIR",
            std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "/tmp/frt-build".to_string()),
        )
        .arg("build")
        .arg("--bin")
        .arg("freenet-example-3")
        .arg("--release");
    let output = cmd
        .output()
        .map_err(|e| format!("spawning cargo build: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cargo build failed:\n{stderr}"));
    }
    let target = target_dir_from_metadata(&game_dir)?;
    let bin = target.join("release").join("freenet-example-3");
    if !bin.exists() {
        return Err(format!("expected build output {} not found", bin.display()));
    }
    Ok(bin)
}

fn target_dir_from_metadata(game_dir: &Path) -> Result<PathBuf, String> {
    let output = Command::new("cargo")
        .current_dir(game_dir)
        .env(
            "CARGO_TARGET_DIR",
            std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "/tmp/frt-build".to_string()),
        )
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .map_err(|e| format!("spawning cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err("cargo metadata failed".to_string());
    }
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("parsing metadata: {e}"))?;
    json.get("target_directory")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .ok_or_else(|| "target_directory missing".to_string())
}

#[cfg(test)]
mod tests {
    use super::build_game;

    #[test]
    fn test_usage() {
        let _ = build_game();
    }
}
