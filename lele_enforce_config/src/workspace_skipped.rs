use std::path::PathBuf;

use derive_more::Deref;

#[derive(Debug, Clone, Deref)]
pub struct WorkspaceSkipped(pub PathBuf);

#[cfg(test)]
mod tests {
    use super::WorkspaceSkipped;
    use std::path::PathBuf;

    #[test]
    fn test_usage() {
        let skipped = WorkspaceSkipped(PathBuf::from("my_workspace"));
        assert_eq!(skipped.display().to_string(), "my_workspace");
    }
}
