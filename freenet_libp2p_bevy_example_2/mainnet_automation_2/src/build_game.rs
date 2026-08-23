use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Error;
use crate::config;
use crate::resolved_bin;

pub fn build_game(cfg: &config::Config) -> Result<PathBuf, Error> {
    let game_dir = game_dir()?;
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&game_dir).arg("build").arg("--workspace");
    if cfg.release {
        cmd.arg("--release");
    }
    let output = cmd
        .output()
        .map_err(|e| Error::Build(format!("spawning cargo build: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Build(format!("cargo build failed:\n{stderr}")));
    }
    resolved_bin::resolved_bin(&game_dir, cfg.release)
}

fn game_dir() -> Result<PathBuf, Error> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| Error::Config("cannot locate game crate directory".to_string()))
}

#[cfg(test)]
mod tests {
    use super::game_dir;

    #[test]
    fn test_usage() {
        assert!(game_dir().is_ok());
    }
}
