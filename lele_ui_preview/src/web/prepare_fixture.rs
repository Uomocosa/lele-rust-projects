use std::path::Path;

use crate::Error;

pub fn prepare_fixture(source: &Path, destination: &Path) -> Result<(), Error> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let target_name = name.strip_suffix(".fixture").unwrap_or(&name);
        let target = destination.join(target_name);
        if entry.file_type()?.is_dir() {
            prepare_fixture(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::prepare_fixture;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("src_fixture");
        std::fs::create_dir_all(source.join("demo/src")).unwrap();
        std::fs::write(source.join("demo/Cargo.toml.fixture"), "[package]").unwrap();
        std::fs::write(source.join("demo/src/lib.rs.fixture"), "pub fn a() {}").unwrap();
        let out = dir.path().join("out");
        prepare_fixture(&source, &out).unwrap();
        assert!(out.join("demo/Cargo.toml").exists());
        assert!(out.join("demo/src/lib.rs").exists());
    }
}
