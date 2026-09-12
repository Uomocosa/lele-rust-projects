use std::path::Path;
use std::path::PathBuf;

use crate::Diagnostic;
use crate::Severity;
use crate::hook_entries;

pub fn check_hook_yamls(root: &Path, crates: &[PathBuf]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for krate in crates {
        let label = krate
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_owned());
        diags.extend(check_dir_yaml(&label, krate));
    }
    diags.extend(check_dir_yaml("root", root));
    diags
}

fn check_dir_yaml(label: &str, dir: &Path) -> Vec<Diagnostic> {
    if !dir.join("devenv.nix").exists() {
        return Vec::new();
    }
    let content = std::fs::read_to_string(dir.join("devenv.nix")).unwrap_or_default();
    if !content.contains("git-hooks") {
        return Vec::new();
    }
    if !dir.join(".pre-commit-config.yaml").exists() {
        return vec![Diagnostic::new(
            label,
            "hook-yaml-missing",
            Severity::Error,
            format!("{label}/.pre-commit-config.yaml not generated"),
            format!(
                "run `devenv shell </dev/null` in {label} to generate it, then merge any new entries into root .pre-commit-config.yaml"
            ),
        )];
    }
    let yaml_text =
        std::fs::read_to_string(dir.join(".pre-commit-config.yaml")).unwrap_or_default();
    let mut diags = Vec::new();
    for cmd in hook_entries(&content) {
        if yaml_text.contains(&cmd) {
            continue;
        }
        diags.push(Diagnostic::new(
            label,
            format!("hook-yaml-stale:{cmd}"),
            Severity::Error,
            format!("hook entry not in {label}/.pre-commit-config.yaml: {cmd}"),
            format!(
                "re-enter `devenv shell </dev/null` in {label}, then merge the entry into root .pre-commit-config.yaml"
            ),
        ));
    }
    diags
}

#[cfg(test)]
mod tests {
    use super::check_hook_yamls;

    fn crate_with(dir: &std::path::Path, nix: &str) -> std::path::PathBuf {
        let krate = dir.join("my_crate");
        std::fs::create_dir_all(&krate).unwrap();
        std::fs::write(krate.join("devenv.nix"), nix).unwrap();
        krate
    }

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let nix = "git-hooks.hooks = {\n  a = {\n    entry = \"bash -c 'cd my_crate && devenv tasks run lele:clippy 2>&1'\";\n  };\n};\n";
        let krate = crate_with(dir.path(), nix);
        let diags = check_hook_yamls(dir.path(), std::slice::from_ref(&krate));
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "hook-yaml-missing");
        std::fs::write(
            krate.join(".pre-commit-config.yaml"),
            "entry: bash -c 'cd my_crate && devenv tasks run lele:clippy 2>&1'\n",
        )
        .unwrap();
        assert!(check_hook_yamls(dir.path(), std::slice::from_ref(&krate)).is_empty());
        let nix2 = format!(
            "{nix}  b = {{\n    entry = \"bash -c 'cd my_crate && devenv tasks run lele:fmt 2>&1'\";\n  }};\n"
        );
        std::fs::write(krate.join("devenv.nix"), nix2).unwrap();
        let diags = check_hook_yamls(dir.path(), &[krate]);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].code.starts_with("hook-yaml-stale"));
    }

    #[test]
    fn test_skips_dirs_without_hooks() {
        let dir = tempfile::tempdir().unwrap();
        let krate = dir.path().join("plain");
        std::fs::create_dir_all(&krate).unwrap();
        assert!(check_hook_yamls(dir.path(), &[krate]).is_empty());
    }
}
