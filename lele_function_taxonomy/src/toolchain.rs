use std::path::{Path, PathBuf};

pub fn sysroot() -> String {
    option_env!("LELE_TAXONOMY_SYSROOT")
        .map(str::to_string)
        .unwrap_or_default()
}

pub fn sysroot_lib() -> PathBuf {
    Path::new(&sysroot()).join("lib")
}

pub fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

// needed helper: the pinned toolchain ships rustc-dev, needed to build the driver
pub fn pinned_cargo() -> PathBuf {
    let candidate = Path::new(&sysroot()).join("bin/cargo");
    if candidate.is_file() {
        candidate
    } else {
        PathBuf::from(cargo())
    }
}

pub fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// needed helper: force cargo to use the pinned rustc from the same sysroot as the driver
pub fn apply_pinned_env(command: &mut std::process::Command) {
    let bin_dir = Path::new(&sysroot()).join("bin");
    let path = {
        let existing = std::env::var("PATH").unwrap_or_default();
        format!("{}:{existing}", bin_dir.display())
    };
    command
        .env("RUSTC", bin_dir.join("rustc"))
        .env("PATH", path);
}

// needed helper: where a self-built driver lands, separate from the analysis target
pub fn driver_target_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".cache/cargo-target/lele_taxonomy_driver")
}

// needed helper: build the driver with the pinned toolchain when it is not found
pub fn build_driver() -> Option<PathBuf> {
    let target_dir = driver_target_dir();
    let mut command = std::process::Command::new(pinned_cargo());
    command
        .arg("build")
        .arg("--manifest-path")
        .arg(manifest_dir().join("Cargo.toml"))
        .arg("--bin")
        .arg("lele-taxonomy-driver")
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("CARGO_BUILD_JOBS", "6");
    apply_pinned_env(&mut command);
    let status = command.status().ok()?;
    if !status.success() {
        return None;
    }
    let candidate = target_dir.join("debug/lele-taxonomy-driver");
    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

pub fn target_dir() -> PathBuf {
    if let Ok(existing) = std::env::var("LELE_TAXONOMY_TARGET_DIR") {
        if !existing.is_empty() {
            return PathBuf::from(existing);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".cache/cargo-target/lele_taxonomy")
}

pub fn driver_path() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("LELE_TAXONOMY_DRIVER") {
        let candidate = PathBuf::from(explicit);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    let built = driver_target_dir().join("debug/lele-taxonomy-driver");
    if built.is_file() {
        return Some(built);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::target_dir;

    #[test]
    fn test_usage() {
        assert!(target_dir().to_string_lossy().contains("lele_taxonomy"));
    }
}
