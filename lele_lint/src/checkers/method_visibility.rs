use std::collections::HashSet;
use std::path::Path;

use crate::checker::{Checker, Diagnostic, Severity};
use crate::config::Config;
use crate::project::{EntryKind, Project};

pub struct MethodVisibility;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("method_visibility") {
        checkers.push(Box::new(MethodVisibility));
    }
}

impl Checker for MethodVisibility {
    fn name(&self) -> &'static str {
        "method_visibility"
    }

    fn code(&self) -> &'static str {
        "E003"
    }

    fn check(&self, project: &Project) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let dir_groups = group_entries_by_parent_dir(&project.entries);

        for (parent_dir, files) in &dir_groups {
            let struct_names = collect_struct_names(files);

            for file_name in files.iter().filter(|f| f.ends_with(".rs")) {
                if let Some(struct_name) = is_method_file(file_name, &struct_names) {
                    if !is_actually_method_file(file_name, parent_dir, project) {
                        continue;
                    }

                    let method_mod_name = file_name.strip_suffix(".rs").unwrap();

                    if let Some(declared_pub) =
                        declared_as_pub_mod(&project.module_info, parent_dir, method_mod_name)
                    {
                        diags.push(Diagnostic {
                            file: declared_pub,
                            line: 1,
                            col: 0,
                            code: "E003".to_string(),
                            message: format!(
                                "method file `{}` of struct `{}` must be declared with `mod` (private), not `pub mod`",
                                file_name, struct_name
                            ),
                            severity: Severity::Error,
                        });
                    }

                    if let Some(reexported_at) =
                        reexported_in_pub_use(&project.module_info, parent_dir, method_mod_name)
                    {
                        diags.push(Diagnostic {
                            file: reexported_at,
                            line: 1,
                            col: 0,
                            code: "E003".to_string(),
                            message: format!(
                                "method file `{}` of struct `{}` must not appear in a `pub use` re-export",
                                file_name, struct_name
                            ),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        diags
    }
}

fn is_actually_method_file(file_name: &str, parent_dir: &str, project: &Project) -> bool {
    let rel_path = if parent_dir.is_empty() {
        Path::new(file_name).to_path_buf()
    } else {
        Path::new(parent_dir).join(file_name)
    };

    let parsed = match project.get_parsed(&rel_path) {
        Some(f) => f,
        None => return true,
    };

    let has_type_definition = parsed
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Struct(_) | syn::Item::Enum(_)));

    !has_type_definition
}

fn group_entries_by_parent_dir(
    entries: &[crate::project::Entry],
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut map: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for entry in entries {
        if entry.kind != EntryKind::File {
            continue;
        }
        let parent = entry
            .relative_path
            .parent()
            .unwrap_or(Path::new(""))
            .to_string_lossy()
            .to_string();
        let file_name = entry
            .relative_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        map.entry(parent).or_default().push(file_name);
    }
    map
}

fn collect_struct_names(file_names: &[String]) -> HashSet<String> {
    let mut names = HashSet::new();
    for f in file_names {
        if let Some(stem) = f.strip_suffix(".rs") {
            if !stem.contains('_') {
                names.insert(stem.to_string());
            }
        }
    }
    names
}

fn is_method_file(file_name: &str, struct_names: &HashSet<String>) -> Option<String> {
    let stem = file_name.strip_suffix(".rs")?;
    if let Some(pos) = stem.find('_') {
        let prefix = &stem[..pos];
        if struct_names.contains(prefix) {
            return Some(prefix.to_string());
        }
    }
    None
}

fn declared_as_pub_mod(
    module_info: &crate::module_info::ModuleInfoMap,
    parent_dir: &str,
    mod_name: &str,
) -> Option<std::path::PathBuf> {
    let mod_rs_path = if parent_dir.is_empty() {
        std::path::PathBuf::from("mod.rs")
    } else {
        std::path::PathBuf::from(parent_dir).join("mod.rs")
    };

    let info = module_info.get(&mod_rs_path)?;

    for decl in &info.declarations {
        if decl.name == mod_name && decl.is_public {
            return Some(info.rel_path.clone());
        }
    }
    None
}

fn reexported_in_pub_use(
    module_info: &crate::module_info::ModuleInfoMap,
    parent_dir: &str,
    mod_name: &str,
) -> Option<std::path::PathBuf> {
    let mod_rs_path = if parent_dir.is_empty() {
        std::path::PathBuf::from("mod.rs")
    } else {
        std::path::PathBuf::from(parent_dir).join("mod.rs")
    };

    let info = module_info.get(&mod_rs_path)?;

    for reexport in &info.reexports {
        if reexport.segments.len() == 1 && reexport.segments[0] == mod_name {
            return Some(info.rel_path.clone());
        }
        if reexport.segments.len() == 2 && reexport.segments[1] == mod_name {
            return Some(info.rel_path.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{collect_struct_names, is_method_file};
    use std::collections::HashSet;

    #[test]
    fn test_usage() {
        let files = vec![
            "player.rs".to_string(),
            "config.rs".to_string(),
            "player_new.rs".to_string(),
            "config_coop.rs".to_string(),
            "spawn.rs".to_string(),
        ];
        let struct_names = collect_struct_names(&files);

        assert_eq!(struct_names.len(), 3);
        assert!(struct_names.contains("player"));
        assert!(struct_names.contains("config"));
        assert!(struct_names.contains("spawn"));

        assert_eq!(
            is_method_file("player_new.rs", &struct_names),
            Some("player".to_string())
        );
        assert_eq!(
            is_method_file("config_coop.rs", &struct_names),
            Some("config".to_string())
        );
        assert_eq!(is_method_file("spawn.rs", &struct_names), None);
        assert_eq!(is_method_file("player.rs", &struct_names), None);
        assert_eq!(is_method_file("bevy_systems.rs", &HashSet::new()), None);
    }
}
