use std::path::Path;

use crate::Diagnostic;
use crate::Severity;

pub fn check_hook_pointer(root: &Path) -> Vec<Diagnostic> {
    if !root.join(".git").exists() {
        return Vec::new();
    }
    let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let expected = root_abs
        .join(".pre-commit-config.yaml")
        .to_string_lossy()
        .to_string();
    let text = std::fs::read_to_string(root.join(".git/hooks/pre-commit")).unwrap_or_default();
    let Some(config) = extract_config(&text) else {
        return vec![Diagnostic::new(
            "root",
            "hook-pointer-missing",
            Severity::Error,
            "git hook not installed: .git/hooks/pre-commit points at no --config",
            format!(
                "point it at the combined config: sed -i 's|--config=\"[^\"]*\"|--config=\"{expected}\"|' .git/hooks/pre-commit"
            ),
        )];
    };
    if config == expected {
        return Vec::new();
    }
    vec![Diagnostic::new(
        "root",
        "hook-pointer-wrong",
        Severity::Error,
        format!("git hook points at {config} instead of the combined {expected}"),
        format!(
            "a crate `devenv shell` stole the pointer; restore it: sed -i 's|--config=\"[^\"]*\"|--config=\"{expected}\"|' .git/hooks/pre-commit"
        ),
    )]
}

fn extract_config(text: &str) -> Option<String> {
    let line = text.lines().find(|line| line.contains("--config="))?;
    let (_, rest) = line.split_once("--config=\"")?;
    let (path, _) = rest.split_once('"')?;
    Some(path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::check_hook_pointer;

    #[test]
    fn test_usage() {
        let bare = tempfile::tempdir().unwrap();
        assert!(check_hook_pointer(bare.path()).is_empty());
        let root = tempfile::tempdir().unwrap();
        let hooks = root.path().join(".git/hooks");
        std::fs::create_dir_all(&hooks).unwrap();
        assert_eq!(check_hook_pointer(root.path()).len(), 1);
        let expected = root
            .path()
            .canonicalize()
            .unwrap()
            .join(".pre-commit-config.yaml")
            .to_string_lossy()
            .to_string();
        std::fs::write(
            hooks.join("pre-commit"),
            format!("exec prek hook-impl --config=\"{expected}\" -- \"$@\"\n"),
        )
        .unwrap();
        assert!(check_hook_pointer(root.path()).is_empty());
        std::fs::write(
            hooks.join("pre-commit"),
            "exec prek hook-impl --config=\"/elsewhere/other.yaml\" -- \"$@\"\n",
        )
        .unwrap();
        let diags = check_hook_pointer(root.path());
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "hook-pointer-wrong");
    }
}
