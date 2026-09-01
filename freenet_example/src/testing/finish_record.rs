use std::path::{Path, PathBuf};
use std::process::Child;

#[must_use]
pub fn finish_record(mut child: Child, path: &Path) -> Option<PathBuf> {
    let _ = child.wait();
    if path.exists() {
        Some(path.to_path_buf())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::finish_record;

    #[test]
    fn test_usage() {
        let _ = finish_record;
    }
}
