use std::path::Path;

use crate::Diagnostic;
use crate::Severity;

fn has_task(content: &str, key: &str) -> bool {
    content.contains(&format!("\"{key}\"")) || content.contains(&format!("'{key}'"))
}

fn has_hook(content: &str, key: &str) -> bool {
    content.contains(key)
}

pub fn check(crate_path: &Path) -> Vec<Diagnostic> {
    let crate_name = crate_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let devenv = crate_path.join("devenv.nix");
    if !devenv.exists() {
        return vec![Diagnostic::new(
            crate_name.clone(),
            "missing-devenv-nix",
            Severity::Error,
            "devenv.nix missing",
            format!(
                "create {crate_name}/devenv.nix from template at ~/.config/opencode/skills/lele-rs/references/lele-rust-config/devenv.nix, replace <crate> with {crate_name}, add devenv.yaml with fenix+git-hooks inputs, then run devenv shell"
            ),
        )];
    }
    let content = std::fs::read_to_string(&devenv).unwrap_or_default();
    let mut diags = Vec::new();
    let tasks: &[(&str, &str)] = &[
        ("lele:build", "cargo build --all-targets"),
        ("lele:clippy", "cargo clippy --all-targets -- -D warnings"),
        ("lele:fmt", "cargo fmt -- --check"),
        ("lele:nextest", "cargo nextest run --all-targets"),
        (
            "lele:lint",
            "cargo run --manifest-path ../lele_lint/Cargo.toml",
        ),
        (
            "lele:taxonomy_check",
            "cargo run --manifest-path ../lele_function_taxonomy/Cargo.toml --features rustc-private -- --manifest-path ./Cargo.toml",
        ),
    ];
    for (key, exec) in tasks {
        if !has_task(&content, key) {
            diags.push(Diagnostic::new(
                crate_name.clone(),
                format!("missing-task:{key}"),
                Severity::Error,
                format!("missing task {key} in devenv.nix"),
                format!(
                    "add tasks.\"{key}\" = {{ exec = \"{exec}\"; showOutput = true; }}; to {crate_name}/devenv.nix (see ~/.config/opencode/skills/lele-rs/references/lele-rust-config/devenv.nix)"
                ),
            ));
        }
    }
    let hooks: &[(&str, &str, &str)] = &[
        ("lele-clippy", "lele:clippy", "clippy"),
        ("lele-fmt", "lele:fmt", "fmt"),
        ("lele-lint", "lele:lint", "lele_lint"),
        ("lele-taxonomy", "lele:taxonomy_check", "taxonomy_check"),
    ];
    for (hook, task, label) in hooks {
        if !has_hook(&content, hook) {
            diags.push(Diagnostic::new(
                crate_name.clone(),
                format!("missing-hook:{hook}"),
                Severity::Error,
                format!("missing git-hook {hook} in devenv.nix"),
                format!(
                    "add git-hooks.hooks.{hook} = {{ enable = true; name = \"{label} ({crate_name})\"; entry = \"bash -c 'cd {crate_name} && devenv tasks run {task} 2>&1'\"; pass_filenames = false; always_run = true; }}; to {crate_name}/devenv.nix then re-enter devenv shell"
                ),
            ));
        }
    }
    diags
}

#[cfg(test)]
mod tests {
    use super::check;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let missing = check(dir.path());
        assert!(!missing.is_empty());
        assert!(missing[0].code.contains("missing-devenv-nix"));
    }

    #[test]
    fn test_with_full_devenv() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"
            tasks."lele:build" = { exec = "cargo build --all-targets"; showOutput = true; };
            tasks."lele:clippy" = { exec = "cargo clippy --all-targets -- -D warnings"; showOutput = true; };
            tasks."lele:fmt" = { exec = "cargo fmt -- --check"; showOutput = true; };
            tasks."lele:nextest" = { exec = "cargo nextest run --all-targets"; showOutput = true; };
            tasks."lele:lint" = { exec = "cargo run --manifest-path ../lele_lint/Cargo.toml"; showOutput = true; };
            tasks."lele:taxonomy_check" = { exec = "cargo run --manifest-path ../lele_function_taxonomy/Cargo.toml --features rustc-private -- --manifest-path ./Cargo.toml"; showOutput = true; };
            git-hooks.hooks.lele-clippy = { enable = true; };
            git-hooks.hooks.lele-fmt = { enable = true; };
            git-hooks.hooks.lele-lint = { enable = true; };
            git-hooks.hooks.lele-taxonomy = { enable = true; };
        "#;
        std::fs::write(dir.path().join("devenv.nix"), content).unwrap();
        let diags = check(dir.path());
        assert!(diags.is_empty());
    }
}
