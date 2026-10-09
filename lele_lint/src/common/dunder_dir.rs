use crate::DunderPath;

pub(crate) fn dunder_dir(dunder_paths: &[DunderPath]) -> Option<String> {
    dunder_paths
        .iter()
        .find(|dunder| dunder.is_dunder_dir())
        .and_then(|dunder| dunder.file_name())
        .and_then(|name| name.to_str())
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::dunder_dir;
    use crate::DunderPath;

    #[test]
    fn test_usage() {
        let file = DunderPath::try_from(PathBuf::from("__prelude__.rs")).unwrap();
        let dir = DunderPath::try_from(PathBuf::from("__basic__")).unwrap();
        assert_eq!(dunder_dir(&[file, dir]).as_deref(), Some("__basic__"));
        assert_eq!(dunder_dir(&[]), None);
    }
}
