use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::method_file_co_location::MethodFileCoLocation;
use crate::common;
use crate::diagnostic;
use crate::project;
use crate::severity;

pub(crate) fn check(
    _self: &MethodFileCoLocation,
    project: &project::Project,
) -> Vec<diagnostic::Diagnostic> {
    let mut diags = Vec::new();

    let type_dirs = build_type_map(project);

    for (rel_path, file) in &project.parsed_files {
        let Some(file_stem) = rel_path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        if common::primary_type_name(file, file_stem).is_some() {
            continue;
        }

        let file_dir = rel_path.parent().unwrap_or(Path::new(""));

        if let Some(candidate_dirs) = find_parent_type(file_stem, &type_dirs) {
            if !candidate_dirs.iter().any(|dir| dir == file_dir) {
                let type_snake = longest_matching_prefix(file_stem, &type_dirs).unwrap();
                let candidates = candidate_dirs
                    .iter()
                    .map(|dir| dir.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                diags.push(diagnostic::Diagnostic {
                    file: project.src_dir.join(rel_path),
                    line: 1,
                    col: 0,
                    code: "E017".to_string(),
                    message: format!(
                        "method file `{}` must be co-located with `{type_snake}.rs`; found in: {candidates}",
                        rel_path.display()
                    ),
                    severity: severity::Severity::Error,
                });
            }
        }
    }

    diags
}

// needed helper: type snake_case → all directories containing it
fn build_type_map(project: &project::Project) -> HashMap<String, Vec<PathBuf>> {
    let mut map = HashMap::<String, Vec<PathBuf>>::new();
    for (rel_path, file) in &project.parsed_files {
        let Some(stem) = rel_path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if common::primary_type_name(file, stem).is_some() {
            let dir = rel_path.parent().unwrap_or(Path::new("")).to_path_buf();
            map.entry(stem.to_string()).or_default().push(dir);
        }
    }
    map
}

// needed helper: find all directories holding a known type matching this stem
fn find_parent_type(stem: &str, type_dirs: &HashMap<String, Vec<PathBuf>>) -> Option<Vec<PathBuf>> {
    longest_matching_prefix(stem, type_dirs).map(|prefix| type_dirs[prefix].clone())
}

// needed helper: longest type prefix matching the stem, if any
fn longest_matching_prefix<'a>(
    stem: &'a str,
    type_dirs: &HashMap<String, Vec<PathBuf>>,
) -> Option<&'a str> {
    let underscores: Vec<usize> = stem.match_indices('_').map(|(i, _)| i).collect();
    for &pos in underscores.iter().rev() {
        let prefix = &stem[..pos];
        if type_dirs.contains_key(prefix) {
            return Some(prefix);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::{find_parent_type, longest_matching_prefix};

    #[test]
    fn test_usage() {
        let mut dirs = HashMap::new();
        dirs.insert("freenet_client".to_string(), vec![PathBuf::from("freenet")]);
        dirs.insert("config".to_string(), vec![PathBuf::from("clicker")]);

        assert_eq!(
            find_parent_type("freenet_client_connect", &dirs),
            Some(vec![PathBuf::from("freenet")])
        );
        assert_eq!(
            find_parent_type("config_new", &dirs),
            Some(vec![PathBuf::from("clicker")])
        );
        assert_eq!(find_parent_type("bevy_systems", &dirs), None);
    }

    #[test]
    fn test_usage_longest_prefix_wins() {
        let mut dirs = HashMap::new();
        dirs.insert("freenet".to_string(), vec![PathBuf::from("freenet")]);
        dirs.insert("freenet_client".to_string(), vec![PathBuf::from("freenet")]);

        assert_eq!(
            longest_matching_prefix("freenet_client_connect", &dirs),
            Some("freenet_client")
        );
    }

    #[test]
    fn test_usage_duplicate_type_dirs() {
        let mut dirs = HashMap::new();
        dirs.insert(
            "plugin".to_string(),
            vec![PathBuf::from("boxes"), PathBuf::from("roster")],
        );

        assert_eq!(
            find_parent_type("plugin_build", &dirs),
            Some(vec![PathBuf::from("boxes"), PathBuf::from("roster")])
        );
    }
}
