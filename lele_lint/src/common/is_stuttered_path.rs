use super::to_snake_case;

pub(crate) fn is_stuttered_path(module: &str, type_name: &str) -> bool {
    type_name.chars().next().is_some_and(|c| c.is_uppercase()) && to_snake_case(type_name) == module
}

#[cfg(test)]
mod tests {
    use super::is_stuttered_path;

    #[test]
    fn test_usage() {
        assert!(is_stuttered_path("diagnostic", "Diagnostic"));
        assert!(is_stuttered_path("entry_kind", "EntryKind"));
        assert!(is_stuttered_path("no_crate_paths", "NoCratePaths"));
    }

    #[test]
    fn test_usage_allows_distinct_names() {
        assert!(!is_stuttered_path("boxes", "PlayerId"));
        assert!(!is_stuttered_path("module_info", "ModuleInfoMap"));
        assert!(!is_stuttered_path("syn", "Item"));
    }

    #[test]
    fn test_usage_allows_function_dispatch() {
        assert!(!is_stuttered_path("config_new", "new"));
        assert!(!is_stuttered_path("walk_entries", "walk_entries"));
    }
}
