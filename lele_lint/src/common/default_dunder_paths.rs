use std::path::PathBuf;

use crate::DunderPath;

pub(crate) fn default_dunder_paths() -> Vec<DunderPath> {
    DunderPath::try_from(PathBuf::from("__basic__"))
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::default_dunder_paths;

    #[test]
    fn test_usage() {
        let dunder_paths = default_dunder_paths();
        assert_eq!(dunder_paths.len(), 1);
        assert_eq!(dunder_paths[0].module_name(), "basic");
    }
}
