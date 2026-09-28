use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use regex::Regex;

use crate::index;
use crate::project;

pub fn discover_projects(roots: &[PathBuf], ignore: &[Regex]) -> Vec<project::ProjectRef> {
    let mut found: Vec<project::ProjectRef> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    for root in roots {
        walk_root(root, ignore, &mut found, &mut seen);
    }
    found.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.root.cmp(&b.root)));
    dedupe_ids(found)
}

// needed helper: find every Cargo.toml directory under a root, skipping ignored and system dirs
fn walk_root(
    root: &PathBuf,
    ignore: &[Regex],
    found: &mut Vec<project::ProjectRef>,
    seen: &mut HashSet<PathBuf>,
) {
    let walker = walkdir::WalkDir::new(root)
        .max_depth(8)
        .into_iter()
        .filter_entry(|entry| !is_skipped(entry) && !is_ignored(entry, ignore));
    for entry in walker.flatten() {
        if entry.file_type().is_file() && entry.file_name().to_str() == Some("Cargo.toml") {
            let Some(dir) = entry.path().parent() else {
                continue;
            };
            let canonical = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
            if !seen.insert(canonical.clone()) {
                continue;
            }
            let name = index::crate_name_of(&canonical);
            found.push(project::ProjectRef {
                id: slugify(&name),
                name,
                root: canonical,
            });
        }
    }
}

// needed helper: directories matching a user ignore rule (settings page) are pruned
fn is_ignored(entry: &walkdir::DirEntry, ignore: &[Regex]) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }
    let path = entry.path().to_string_lossy();
    ignore.iter().any(|rule| rule.is_match(&path))
}

// needed helper: system directories that should never be descended into
fn is_skipped(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }
    matches!(
        entry.file_name().to_str(),
        Some(
            ".devenv"
                | ".rustup"
                | ".cargo"
                | ".cache"
                | ".local"
                | ".nix-defexpr"
                | ".nix-profile"
        )
    )
}

// needed helper: stable url-safe slug from a crate name
fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "crate".to_string()
    } else {
        trimmed
    }
}

// needed helper: make project ids unique by suffixing duplicates
fn dedupe_ids(found: Vec<project::ProjectRef>) -> Vec<project::ProjectRef> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut used: HashSet<String> = HashSet::new();
    let mut out = Vec::with_capacity(found.len());
    for mut item in found {
        let base = slugify(&item.name);
        let mut n = counts.get(&base).copied().unwrap_or(0).saturating_add(1);
        let mut id = if n == 1 {
            base.clone()
        } else {
            format!("{base}-{n}")
        };
        while !used.insert(id.clone()) {
            n = n.saturating_add(1);
            id = format!("{base}-{n}");
        }
        counts.insert(base, n);
        item.id = id;
        out.push(item);
    }
    out
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::discover_projects;
    use crate::project;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("alpha");
        let b = dir.path().join("beta");
        fs::create_dir_all(a.join("src")).unwrap();
        fs::create_dir_all(b.join("src")).unwrap();
        fs::write(
            a.join("Cargo.toml"),
            "[package]\nname = \"alpha\"\nversion = \"0.0.0\"\n",
        )
        .unwrap();
        fs::write(
            b.join("Cargo.toml"),
            "[package]\nname = \"alpha\"\nversion = \"0.0.0\"\n",
        )
        .unwrap();
        let projects = discover_projects(&[dir.path().to_path_buf()], &[]);
        assert_eq!(projects.len(), 2);
        assert_ne!(projects[0].id, projects[1].id);
    }

    #[test]
    fn test_ignore_rules_prune_folders() {
        let dir = tempfile::tempdir().unwrap();
        for folder in ["keep", "__OLD__/old", "keep/target/built"] {
            let crate_dir = dir.path().join(folder);
            fs::create_dir_all(&crate_dir).unwrap();
            fs::write(
                crate_dir.join("Cargo.toml"),
                "[package]\nname = \"demo\"\nversion = \"0.0.0\"\n",
            )
            .unwrap();
        }
        let rules = project::compile_rules(&project::default_settings().ignore).unwrap();
        let projects = discover_projects(&[dir.path().to_path_buf()], &rules);
        assert_eq!(projects.len(), 1);
        assert!(projects[0].root.ends_with("keep"));
        let everything = discover_projects(&[dir.path().to_path_buf()], &[]);
        assert_eq!(everything.len(), 3);
    }
}
