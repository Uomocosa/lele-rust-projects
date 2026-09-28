use std::path::Path;

pub fn display_path(path: &Path) -> String {
    let full = path.to_string_lossy().to_string();
    let Some(home) = std::env::var_os("HOME") else {
        return full;
    };
    let home = home.to_string_lossy().to_string();
    match full.strip_prefix(&home) {
        Some(rest) if !home.is_empty() && (rest.is_empty() || rest.starts_with('/')) => {
            format!("~{rest}")
        }
        _ => full,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::display_path;

    #[test]
    fn test_usage() {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap());
        assert_eq!(display_path(&home.join("code/demo")), "~/code/demo");
        assert_eq!(display_path(&PathBuf::from("/opt/demo")), "/opt/demo");
    }
}
