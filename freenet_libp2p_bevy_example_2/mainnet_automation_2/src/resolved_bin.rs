use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Error;

const BIN_NAME: &str = "freenet-libp2p-bevy-example-2";

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

#[cfg(test)]
mod tests {
    use super::BIN_NAME;

    #[test]
    fn test_usage() {
        assert!(BIN_NAME.ends_with("example-2"));
    }
}
