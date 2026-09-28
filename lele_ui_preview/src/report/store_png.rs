use std::path::Path;

use crate::Error;
use crate::report;

pub fn store_png(
    out_dir: &Path,
    group: &str,
    screen: &str,
    id: &str,
    label: &str,
    bytes: &[u8],
) -> Result<String, Error> {
    let relative = format!(
        "{}/{}/{id}__{}.png",
        report::slug(group),
        report::slug(screen),
        report::slug(label)
    );
    let path = out_dir.join(&relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, bytes)?;
    Ok(relative)
}

#[cfg(test)]
mod tests {
    use super::store_png;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let rel = store_png(dir.path(), "mobile", "/p/{id}", "s001", "open menu", b"x").unwrap();
        assert_eq!(rel, "mobile/p-id/s001__open-menu.png");
        assert!(dir.path().join(rel).exists());
    }
}
