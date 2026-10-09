use std::collections::HashSet;
use std::path::Path;

use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::Entry;
use crate::EntryKind;
use crate::ModuleInfoMap;
use crate::Project;

pub fn check(
    _self: &checkers::method_visibility::MethodVisibility,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let dir_groups = group_entries_by_parent_dir(&project.entries);

    for (parent_dir, files) in &dir_groups {
        let struct_names = collect_struct_names(files, parent_dir, project);

        for file_name in files.iter().filter(|f| f.ends_with(".rs")) {
            let Some(struct_name) = file_name
                .strip_suffix(".rs")
                .and_then(|stem| common::longest_type_prefix(stem, &struct_names))
                .map(str::to_string)
            else {
                continue;
            };
            if !is_actually_method_file(file_name, parent_dir, project) {
                continue;
            }

            let method_mod_name = file_name.strip_suffix(".rs").unwrap_or(file_name);

            if let Some(declared_pub) =
                declared_as_pub_mod(&project.module_info, parent_dir, method_mod_name)
            {
                diags.push(Diagnostic {
                    file: declared_pub,
                    line: 1,
                    col: 0,
                    code: checkers::method_visibility::MethodVisibility::CODE,
                    message: format!(
                        "method file `{}` of struct `{}` must be declared with `mod` (private), not `pub mod`",
                        file_name, struct_name
                    ),
                });
            }

            if let Some(reexported_at) =
                reexported_in_pub_use(&project.module_info, parent_dir, method_mod_name)
            {
                diags.push(Diagnostic {
                    file: reexported_at,
                    line: 1,
                    col: 0,
                    code: checkers::method_visibility::MethodVisibility::CODE,
                    message: format!(
                        "method file `{}` of struct `{}` must not appear in a `pub use` re-export",
                        file_name, struct_name
                    ),
                });
            }
        }
    }

    diags
}

// needed helper: method-file classification (no type definition)
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

    if has_type_definition {
        return false;
    }

    parsed
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Fn(_)))
}

// needed helper: directory-grouped entry map
fn group_entries_by_parent_dir(
    entries: &[Entry],
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
        let Some(file_name) = entry.relative_path.file_name() else {
            continue;
        };
        let file_name = file_name.to_string_lossy().to_string();
        map.entry(parent).or_default().push(file_name);
    }
    map
}

// needed helper: struct name set from file listing via AST type detection
fn collect_struct_names(
    file_names: &[String],
    parent_dir: &str,
    project: &Project,
) -> HashSet<String> {
    let mut names = HashSet::new();
    for f in file_names {
        let Some(stem) = f.strip_suffix(".rs") else {
            continue;
        };
        let rel_path = if parent_dir.is_empty() {
            Path::new(f).to_path_buf()
        } else {
            Path::new(parent_dir).join(f)
        };
        let Some(parsed) = project.get_parsed(&rel_path) else {
            continue;
        };
        if common::primary_type_name(parsed, stem).is_some() {
            names.insert(stem.to_string());
        }
    }
    names
}

// needed helper: pub mod declaration check
fn declared_as_pub_mod(
    module_info: &ModuleInfoMap,
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

// needed helper: pub use re-export check
fn reexported_in_pub_use(
    module_info: &ModuleInfoMap,
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
        if reexport.segments.len() == 1 && reexport.segments.first().is_some_and(|s| s == mod_name)
        {
            return Some(info.rel_path.clone());
        }
        if reexport.segments.len() == 2 && reexport.segments.get(1).is_some_and(|s| s == mod_name) {
            return Some(info.rel_path.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::common;

    fn method_struct(file_name: &str, struct_names: &HashSet<String>) -> Option<String> {
        file_name
            .strip_suffix(".rs")
            .and_then(|stem| common::longest_type_prefix(stem, struct_names))
            .map(str::to_string)
    }

    #[test]
    fn test_usage() {
        let mut struct_names = HashSet::new();
        struct_names.insert("player".to_string());
        struct_names.insert("config".to_string());
        struct_names.insert("freenet_client".to_string());

        assert_eq!(
            method_struct("player_new.rs", &struct_names),
            Some("player".to_string())
        );
        assert_eq!(
            method_struct("freenet_client_connect.rs", &struct_names),
            Some("freenet_client".to_string())
        );
        assert_eq!(method_struct("bevy_systems.rs", &struct_names), None);
    }
}
