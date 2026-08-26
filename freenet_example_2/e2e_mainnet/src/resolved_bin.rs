use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Error;

const BIN_NAME: &str = "freenet-example-2";

pub fn resolved_bin(game_dir: &Path, release: bool) -> Result<PathBuf, Error> {
    let target = metadata_target_dir(game_dir)?;
    let profile = if release { "release" } else { "debug" };
    let bin = target.join(profile).join(BIN_NAME);
    if !bin.exists() {
        return Err(Error::Build(format!(
            "expected build output {} not found",
            bin.display()
        )));
    }
    Ok(bin)
}

fn metadata_target_dir(game_dir: &Path) -> Result<PathBuf, Error> {
    let output = Command::new("cargo")
        .current_dir(game_dir)
        .env("CARGO_TARGET_DIR", target_dir())
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .map_err(|e| Error::Metadata(format!("spawning cargo metadata: {e}")))?;
    if !output.status.success() {
        return Err(Error::Metadata("cargo metadata failed".to_string()));
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| Error::Metadata(format!("parsing cargo metadata json: {e}")))?;
    json["target_directory"]
        .as_str()
        .map(PathBuf::from)
        .ok_or_else(|| Error::Metadata("target_directory missing in cargo metadata".to_string()))
}

fn target_dir() -> String {
    std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "/tmp/frt-build".to_string())
}

#[cfg(test)]
mod tests {
    use super::BIN_NAME;

    #[test]
    fn test_usage() {
        assert_eq!(BIN_NAME, "freenet-example-2");
    }
}
