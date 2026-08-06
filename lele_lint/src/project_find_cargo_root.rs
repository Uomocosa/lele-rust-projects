use crate::error::Error;
use std::path::{Path, PathBuf};

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

// no test_usage necessary
