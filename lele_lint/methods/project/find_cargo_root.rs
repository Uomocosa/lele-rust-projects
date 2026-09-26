use std::path::{Path, PathBuf};

use crate::Error;

pub fn find_cargo_root(start: &Path) -> Result<PathBuf, Error> {
    let mut current = start.to_path_buf();
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(current);
        }
        if !current.pop() {
            return Err(Error::NoCargoRoot(start.display().to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::find_cargo_root;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        assert_eq!(
            find_cargo_root(dir.path()).unwrap(),
            dir.path().to_path_buf()
        );
        assert_eq!(
            find_cargo_root(&dir.path().join("nested")).unwrap(),
            dir.path().to_path_buf()
        );
    }
}
