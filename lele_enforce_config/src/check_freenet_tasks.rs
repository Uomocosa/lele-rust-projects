use std::path::Path;

use crate::Diagnostic;
use crate::Severity;

fn has_task(content: &str, key: &str) -> bool {
    content.contains(&format!("\"{key}\"")) || content.contains(&format!("'{key}'"))
}

fn has_all(content: &str, needles: &[&str]) -> bool {
    needles.iter().all(|needle| content.contains(needle))
}

pub fn check_freenet_tasks(crate_path: &Path) -> Vec<Diagnostic> {
    let crate_name = crate_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let devenv = crate_path.join("devenv.nix");
    let content = std::fs::read_to_string(&devenv).unwrap_or_default();
    let mut diags = Vec::new();
    let tasks: &[(&str, &str, &[&str])] = &[
        (
            "freenet:contract-harness",
            "cargo test --manifest-path ../freenet_contract_harness/Cargo.toml -- --nocapture",
            &[],
        ),
        (
            "freenet:run-local-mainnet",
            "cargo nextest run --test mainnet_local --all-features --run-ignored all -- --nocapture",
            &["--all-features"],
        ),
        (
            "freenet:run-cross-os",
            "cargo nextest run --test mainnet_cross --all-features --run-ignored all -- --nocapture",
            &["--all-features"],
        ),
    ];
    for (key, exec, needles) in tasks {
        if !has_task(&content, key) {
            diags.push(Diagnostic::new(
                crate_name.clone(),
                format!("missing-task:{key}"),
                Severity::Error,
                format!("freenet crate missing task {key} in devenv.nix"),
                format!(
                    "add tasks.\"{key}\" = {{ exec = \"{exec}\"; showOutput = true; }}; to {crate_name}/devenv.nix (see freenet_example/devenv.nix)"
                ),
            ));
        } else if !has_all(&content, needles) {
            diags.push(Diagnostic::new(
                crate_name.clone(),
                format!("missing-flags:{key}"),
                Severity::Error,
                format!("freenet task {key} must run with {}", needles.join(" and ")),
                format!("set tasks.\"{key}\".exec = \"{exec}\"; in {crate_name}/devenv.nix"),
            ));
        }
    }
    diags
}

#[cfg(test)]
mod tests {
    use super::check_freenet_tasks;
    use crate::uses_freenet;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[dependencies]\nfreenet = \"0.1\"\n",
        )
        .unwrap();
        assert!(uses_freenet(dir.path()));
        let diags = check_freenet_tasks(dir.path());
        assert_eq!(diags.len(), 3);
    }
}
