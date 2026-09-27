use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

pub fn resolve_in_root(root: &Path, rel: &Path) -> Option<PathBuf> {
    if rel.as_os_str().is_empty() {
        return None;
    }
    if !rel
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        return None;
    }
    let root = root.canonicalize().ok()?;
    let full = root.join(rel).canonicalize().ok()?;
    if !full.starts_with(&root) || !full.is_file() {
        return None;
    }
    Some(full)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::resolve_in_root;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("proj");
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("docs/a.md"), "# a\n").unwrap();
        fs::write(dir.path().join("secret.md"), "no\n").unwrap();
        assert!(resolve_in_root(&root, Path::new("docs/a.md")).is_some());
        assert!(resolve_in_root(&root, Path::new("../secret.md")).is_none());
        assert!(resolve_in_root(&root, Path::new("docs/../../secret.md")).is_none());
        assert!(resolve_in_root(&root, Path::new("/etc/passwd")).is_none());
        assert!(resolve_in_root(&root, Path::new("docs")).is_none());
        assert!(resolve_in_root(&root, Path::new("")).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn test_symlink_escape_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("proj");
        fs::create_dir_all(&root).unwrap();
        fs::write(dir.path().join("secret.md"), "no\n").unwrap();
        std::os::unix::fs::symlink(dir.path().join("secret.md"), root.join("link.md")).unwrap();
        assert!(resolve_in_root(&root, Path::new("link.md")).is_none());
    }
}
