use std::path::Path;

pub(crate) fn module_path_of(rel_path: &Path) -> Vec<String> {
    let mut components: Vec<String> = rel_path
        .components()
        .filter_map(|c| c.as_os_str().to_str().map(str::to_string))
        .collect();
    let Some(file_name) = components.pop() else {
        return Vec::new();
    };
    if is_index_file(&file_name) {
        return components;
    }
    let stem = file_name.strip_suffix(".rs").unwrap_or(&file_name);
    components.push(stem.to_string());
    components
}

// needed helper: crate root or directory index files map to their parent module
fn is_index_file(file_name: &str) -> bool {
    matches!(file_name, "mod.rs" | "lib.rs" | "main.rs")
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
