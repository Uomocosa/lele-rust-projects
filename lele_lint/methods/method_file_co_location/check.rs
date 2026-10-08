use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::Project;

pub fn check(
    _self: &checkers::method_file_co_location::MethodFileCoLocation,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let type_dirs = build_type_map(project);
    let known_stems: HashSet<String> = type_dirs.keys().cloned().collect();

    for (rel_path, file) in &project.parsed_files {
        let Some(file_stem) = rel_path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        if common::primary_type_name(file, file_stem).is_some() {
            continue;
        }

        let file_dir = rel_path.parent().unwrap_or(Path::new(""));

        if let Some(candidate_dirs) = find_parent_type(file_stem, &known_stems, &type_dirs) {
            if !candidate_dirs.iter().any(|dir| dir == file_dir) {
                let Some(type_snake) = common::longest_type_prefix(file_stem, &known_stems) else {
                    continue;
                };
                let candidates = candidate_dirs
                    .iter()
                    .map(|dir| dir.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                diags.push(Diagnostic {
                    file: project.src_dir.join(rel_path),
                    line: 1,
                    col: 0,
                    code: "E017".to_string(),
                    message: format!(
                        "method file `{}` must be co-located with `{type_snake}.rs`; found in: {candidates}",
                        rel_path.display()
                    ),
                });
            }
        } else if let Some(suffix) = file_stem.rsplit('_').next() {
            if file_stem.contains('_') && pub_fn_names(file).iter().any(|n| n == suffix) {
                diags.push(Diagnostic {
                    file: project.src_dir.join(rel_path),
                    line: 1,
                    col: 0,
                    code: "E017".to_string(),
                    message: format!(
                        "orphan method file `{file_stem}.rs` — no parent type found; rename to `{suffix}.rs` or `<type>_{suffix}.rs` with a `<type>.rs` defining the type",
                    ),
                });
            }
        }
    }

    diags
}

// needed helper: public free-function names in a file
fn pub_fn_names(file: &syn::File) -> Vec<String> {
    file.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Fn(f) if matches!(f.vis, syn::Visibility::Public(_)) => {
                Some(f.sig.ident.to_string())
            }
            _ => None,
        })
        .collect()
}

// needed helper: type snake_case → all directories containing it
fn build_type_map(project: &Project) -> HashMap<String, Vec<PathBuf>> {
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
fn find_parent_type(
    stem: &str,
    known_stems: &HashSet<String>,
    type_dirs: &HashMap<String, Vec<PathBuf>>,
) -> Option<Vec<PathBuf>> {
    common::longest_type_prefix(stem, known_stems).and_then(|prefix| type_dirs.get(prefix).cloned())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::{check, find_parent_type, pub_fn_names};
    use crate::checkers::method_file_co_location::MethodFileCoLocation;
    use crate::common;
    use crate::Project;

    fn known_stems(dirs: &HashMap<String, Vec<PathBuf>>) -> HashSet<String> {
        dirs.keys().cloned().collect()
    }

    #[test]
    fn test_usage() {
        let mut dirs = HashMap::new();
        dirs.insert("freenet_client".to_string(), vec![PathBuf::from("freenet")]);
        dirs.insert("config".to_string(), vec![PathBuf::from("clicker")]);

        assert_eq!(
            find_parent_type("freenet_client_connect", &known_stems(&dirs), &dirs),
            Some(vec![PathBuf::from("freenet")])
        );
        assert_eq!(
            find_parent_type("config_new", &known_stems(&dirs), &dirs),
            Some(vec![PathBuf::from("clicker")])
        );
        assert_eq!(
            find_parent_type("bevy_systems", &known_stems(&dirs), &dirs),
            None
        );
    }

    #[test]
    fn test_usage_longest_prefix_wins() {
        let mut dirs = HashMap::new();
        dirs.insert("freenet".to_string(), vec![PathBuf::from("freenet")]);
        dirs.insert("freenet_client".to_string(), vec![PathBuf::from("freenet")]);

        assert_eq!(
            common::longest_type_prefix("freenet_client_connect", &known_stems(&dirs)),
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
            find_parent_type("plugin_build", &known_stems(&dirs), &dirs),
            Some(vec![PathBuf::from("boxes"), PathBuf::from("roster")])
        );
    }

    #[test]
    fn test_usage_pub_fn_names() {
        let file: syn::File =
            syn::parse_str("pub fn load() {}\nfn private() {}\npub struct Helper;").unwrap();
        assert_eq!(pub_fn_names(&file), vec!["load".to_string()]);
    }

    fn orphan_project(files: &[(&str, &str)]) -> Project {
        let mut parsed_files = std::collections::HashMap::new();
        for (name, code) in files {
            parsed_files.insert(PathBuf::from(name), syn::parse_str(code).unwrap());
        }
        Project {
            root: PathBuf::from("."),
            src_dir: PathBuf::from("src"),
            entries: Vec::new(),
            module_info: std::collections::HashMap::default(),
            parsed_files,
            ..Project::default()
        }
    }

    #[test]
    fn test_usage_orphan_method_file_flagged() {
        let project = orphan_project(&[
            ("lele_config.rs", "pub struct LeleConfig { pub x: u8 }"),
            ("config_load.rs", "pub fn load() {}"),
        ]);
        let diags = check(&MethodFileCoLocation, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("orphan method file"));
    }

    #[test]
    fn test_usage_plain_free_function_passes() {
        let project = orphan_project(&[
            ("lele_config.rs", "pub struct LeleConfig { pub x: u8 }"),
            ("load.rs", "pub fn load() {}"),
        ]);
        let diags = check(&MethodFileCoLocation, &project);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_exact_match_multi_underscore_passes() {
        let project = orphan_project(&[(
            "to_snake_case.rs",
            "pub fn to_snake_case(input: &str) -> String { input.to_string() }",
        )]);
        let diags = check(&MethodFileCoLocation, &project);
        assert!(diags.is_empty());
    }
}
