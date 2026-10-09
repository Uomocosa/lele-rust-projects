use std::path::PathBuf;

use crate::DunderPath;
use crate::Error;

pub fn try_from(path: PathBuf) -> Result<DunderPath, Error> {
    let is_dunder = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.strip_prefix("__"))
        .and_then(|rest| rest.strip_suffix("__"))
        .is_some_and(|inner| !inner.is_empty());
    if !is_dunder {
        return Err(Error::NotDunder(path));
    }
    Ok(DunderPath(path))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::DunderPath;

    fn parses(raw: &str) -> bool {
        DunderPath::try_from(PathBuf::from(raw)).is_ok()
    }

    #[test]
    fn test_usage() {
        assert!(parses("__basic__"));
        assert!(parses("__prelude__.rs"));
        assert!(parses("__notes__.md"));
        assert!(parses("src/__basic__"));
    }

    #[test]
    fn test_usage_rejects_non_dunder_names() {
        assert!(!parses("basic"));
        assert!(!parses("__basic"));
        assert!(!parses("basic__"));
        assert!(!parses("____"));
        assert!(!parses("____.rs"));
        assert!(!parses("__basic__/enums.rs"));
    }
}
