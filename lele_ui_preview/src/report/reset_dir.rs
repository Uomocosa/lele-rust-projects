use std::path::Path;

use crate::Error;

pub fn reset_dir(out_dir: &Path) -> Result<(), Error> {
    if out_dir.exists() {
        std::fs::remove_dir_all(out_dir)?;
    }
    std::fs::create_dir_all(out_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::reset_dir;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("ui_preview");
        std::fs::create_dir_all(out.join("old")).unwrap();
        reset_dir(&out).unwrap();
        assert!(out.exists());
        assert!(!out.join("old").exists());
    }
}
