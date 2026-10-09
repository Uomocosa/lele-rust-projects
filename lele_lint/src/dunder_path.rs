use std::path::PathBuf;

use atomic_delegate_macros::atomic_delegate;
use derive_more::Deref;
use serde::Deserialize;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deref, Deserialize)]
#[serde(try_from = "PathBuf")]
pub struct DunderPath(pub(crate) PathBuf);

#[rustfmt::skip]
impl DunderPath {
    pub fn module_name(&self) -> &str {
        self.file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|stem| stem.strip_prefix("__"))
            .and_then(|rest| rest.strip_suffix("__"))
            .unwrap_or_default()
    }
    pub fn is_dunder_dir(&self) -> bool { self.extension().is_none() }
    pub fn is_dunder_file(&self) -> bool { self.extension().is_some() }
}

#[rustfmt::skip]
impl TryFrom<PathBuf> for DunderPath {
    type Error = Error;
    #[atomic_delegate(DunderPath)]
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {}
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::DunderPath;

    #[test]
    fn test_usage() {
        let dir = DunderPath::try_from(PathBuf::from("__basic__")).unwrap();
        assert_eq!(dir.module_name(), "basic");
        assert!(dir.is_dunder_dir());
        assert!(!dir.is_dunder_file());

        let file = DunderPath::try_from(PathBuf::from("src/__notes__.md")).unwrap();
        assert_eq!(file.module_name(), "notes");
        assert!(file.is_dunder_file());
        assert!(!file.is_dunder_dir());
    }
}
