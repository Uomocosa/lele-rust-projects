use std::path::Path;

use crate::Diagnostic;
use crate::Error;
use crate::LeleConfig;
use crate::WorkspaceSkipped;
use crate::check;
use crate::check_freenet_tasks;
use crate::discover;
use crate::is_workspace;
use crate::uses_freenet;

/// # Errors
///
/// Returns [`Error`] when `lele.toml` exists but fails to parse as TOML.
pub fn run(root: &Path) -> Result<(Vec<Diagnostic>, Vec<WorkspaceSkipped>), Error> {
    let cfg = LeleConfig::load(root)?;
    let crates = discover(root, &cfg);
    let workspaces_skipped = collect_workspaces(root, &cfg);
    let mut diags = Vec::new();
    for krate in &crates {
        let mut d = check(krate);
        let has_devenv = krate.join("devenv.nix").exists();
        if has_devenv && uses_freenet(krate) {
            d.extend(check_freenet_tasks(krate));
        }
        diags.extend(d);
    }
    Ok((diags, workspaces_skipped))
}

fn collect_workspaces(root: &Path, exclude: &LeleConfig) -> Vec<WorkspaceSkipped> {
    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut skipped = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if exclude.is_excluded(root, &path) {
            continue;
        }
        if !path.is_dir() || !path.join("Cargo.toml").exists() {
            continue;
        }
        if is_workspace(&path) {
            skipped.push(WorkspaceSkipped(path));
        }
    }
    skipped
}

#[cfg(test)]
mod tests {
    use super::run;
    use std::path::Path;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let (diags, skipped) = run(dir.path()).unwrap();
        assert!(diags.is_empty());
        assert!(skipped.is_empty());
        let _ = Path::new("/tmp");
    }
}
