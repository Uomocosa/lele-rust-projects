use std::path::Path;

use crate::common;

pub(crate) fn module_path_of(rel_path: &Path) -> Vec<String> {
    let mut components: Vec<String> = rel_path
        .components()
        .filter_map(|c| c.as_os_str().to_str().map(str::to_string))
        .collect();
    let Some(file_name) = components.pop() else {
        return Vec::new();
    };
    if common::is_index_file(&file_name) {
        return components;
    }
    let stem = file_name.strip_suffix(".rs").unwrap_or(&file_name);
    components.push(stem.to_string());
    components
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::module_path_of;

    #[test]
    fn test_usage() {
        assert_eq!(
            module_path_of(&PathBuf::from("a/mod.rs")),
            vec!["a".to_string()]
        );
        assert_eq!(
            module_path_of(&PathBuf::from("a/b.rs")),
            vec!["a".to_string(), "b".to_string()]
        );
    }
}
