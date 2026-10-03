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
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let candidate = dir.join("lele-taxonomy-driver");
    if candidate.is_file() {
        return Some(candidate);
    }
    let sibling = dir.join("lele-taxonomy-driver.exe");
    if sibling.is_file() {
        return Some(sibling);
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
