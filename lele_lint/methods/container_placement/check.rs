use std::path::{Path, PathBuf};

use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::Project;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Resource,
    Component,
    Message,
    Event,
    Newtype,
    Struct,
    Enum,
    TypeAlias,
    Const,
}

impl Role {
    // no test_usage necessary
    fn file(self) -> &'static str {
        match self {
            Role::Resource => "resources.rs",
            Role::Component => "components.rs",
            Role::Message => "messages.rs",
            Role::Event => "events.rs",
            Role::Newtype => "newtypes.rs",
            Role::Struct => "structs.rs",
            Role::Enum => "enums.rs",
            Role::TypeAlias => "type_aliases.rs",
            Role::Const => "constants.rs",
        }
    }
}

pub fn check(
    _self: &checkers::container_placement::ContainerPlacement,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let Some(folder) = common::dunder_dir(&project.dunder_paths) else {
        return diags;
    };

    for source in project.content_sources() {
        let rel_path = source.relative_path;
        let file = source.file;
        let file_path = project.absolute_path(source.origin, rel_path);
        if common::in_dunder_dir(rel_path, &project.dunder_paths) {
            check_container_file(rel_path, file, project, &file_path, &mut diags);
        } else {
            check_free_file(rel_path, file, &folder, &file_path, &mut diags);
        }
    }

    diags
}

// needed helper: role named by a container file stem
fn role_of_stem(stem: &str) -> Option<Role> {
    match stem {
        "resources" => Some(Role::Resource),
        "components" => Some(Role::Component),
        "messages" => Some(Role::Message),
        "events" => Some(Role::Event),
        "newtypes" => Some(Role::Newtype),
        "structs" => Some(Role::Struct),
        "enums" => Some(Role::Enum),
        "type_aliases" => Some(Role::TypeAlias),
        "constants" => Some(Role::Const),
        _ => None,
    }
}

// needed helper: container file item-role validation
fn check_container_file(
    rel_path: &Path,
    file: &syn::File,
    project: &Project,
    file_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    let Some(stem) = rel_path.file_stem().and_then(|s| s.to_str()) else {
        return;
    };
    if stem == "mod.rs" || common::is_dunder_file_path(rel_path, &project.dunder_paths) {
        return;
    }
    let Some(role) = role_of_stem(stem) else {
        return;
    };

    for item in &file.items {
        let Some(name) = exposed_name(item) else {
            continue;
        };
        if has_behavior(file, &name) {
            push(
                diags,
                file_path,
                format!(
                    "`{name}` has impls; `{stem}.rs` holds behavior-free items only — move it to `{name}.rs`"
                ),
            );
            continue;
        }
        let natural = classify(item);
        if natural != Some(role) {
            let hint = natural.map_or("a non-container item".to_string(), |r| r.file().to_string());
            push(
                diags,
                file_path,
                format!("`{name}` is a {hint}; move it to `{hint}`"),
            );
        }
    }
}

// needed helper: free-file qualification -> container placement diagnostics
fn check_free_file(
    rel_path: &Path,
    file: &syn::File,
    folder: &str,
    file_path: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    let Some(stem) = rel_path.file_stem().and_then(|s| s.to_str()) else {
        return;
    };
    if stem == "mod.rs" || stem == "lib.rs" {
        return;
    }
    let target = container_dir(rel_path, folder);

    for item in &file.items {
        let Some(name) = exposed_name(item) else {
            continue;
        };
        if has_behavior(file, &name) {
            continue;
        }
        let Some(role) = classify(item) else {
            continue;
        };
        if role == Role::Const {
            continue;
        }
        push(
            diags,
            file_path,
            format!(
                "`{name}` is a behavior-free {}; move it to `{}`",
                role.file(),
                target.join(role.file()).display(),
            ),
        );
    }
}

// needed helper: diagnostic construction
fn push(diags: &mut Vec<Diagnostic>, file_path: &Path, message: String) {
    diags.push(Diagnostic {
        file: file_path.to_path_buf(),
        line: 1,
        col: 0,
        code: checkers::container_placement::ContainerPlacement::CODE,
        message,
    });
}

// needed helper: exposed (pub/pub(crate)) primary item name
fn exposed_name(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Struct(s) if common::is_exposed(&s.vis) => Some(s.ident.to_string()),
        syn::Item::Enum(e) if common::is_exposed(&e.vis) => Some(e.ident.to_string()),
        syn::Item::Type(t) if common::is_exposed(&t.vis) => Some(t.ident.to_string()),
        syn::Item::Const(c) if common::is_exposed(&c.vis) => Some(c.ident.to_string()),
        syn::Item::Static(s) if common::is_exposed(&s.vis) => Some(s.ident.to_string()),
        _ => None,
    }
}

// needed helper: role a free item would occupy if containerized
fn classify(item: &syn::Item) -> Option<Role> {
    match item {
        syn::Item::Struct(s) => {
            let derives = derive_names(&s.attrs);
            if is_excluded(&derives) {
                return None;
            }
            if let Some(role) = ecs_role(&derives) {
                return Some(role);
            }
            if matches!(s.fields, syn::Fields::Unnamed(ref f) if f.unnamed.len() == 1) {
                Some(Role::Newtype)
            } else {
                Some(Role::Struct)
            }
        }
        syn::Item::Enum(e) => {
            let derives = derive_names(&e.attrs);
            if is_excluded(&derives) {
                return None;
            }
            ecs_role(&derives).or(Some(Role::Enum))
        }
        syn::Item::Type(_) => Some(Role::TypeAlias),
        syn::Item::Const(_) | syn::Item::Static(_) => Some(Role::Const),
        _ => None,
    }
}

// needed helper: ECS marker derive -> role
fn ecs_role(derives: &[String]) -> Option<Role> {
    for derive in derives {
        match derive.as_str() {
            "Resource" => return Some(Role::Resource),
            "Component" => return Some(Role::Component),
            "Message" => return Some(Role::Message),
            "Event" => return Some(Role::Event),
            _ => {}
        }
    }
    None
}

// needed helper: derives that force a type to stay atomic
fn is_excluded(derives: &[String]) -> bool {
    derives.iter().any(|derive| {
        matches!(
            derive.as_str(),
            "Error" | "Parser" | "Args" | "Subcommand" | "ValueEnum" | "NetworkBehaviour"
        )
    })
}

// needed helper: derive attribute name extraction
fn derive_names(attrs: &[syn::Attribute]) -> Vec<String> {
    let mut names = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }
        let parsed = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
        );
        if let Ok(paths) = parsed {
            for path in paths {
                if let Some(segment) = path.segments.last() {
                    names.push(segment.ident.to_string());
                }
            }
        }
    }
    names
}

// needed helper: any impl (inherent/trait/atomic-delegate) for the type
fn has_behavior(file: &syn::File, type_name: &str) -> bool {
    file.items.iter().any(|item| {
        let syn::Item::Impl(impl_block) = item else {
            return false;
        };
        if common::has_atomic_delegates(&impl_block.attrs) {
            return true;
        }
        common::self_type_last(&impl_block.self_ty).as_deref() == Some(type_name)
    })
}

// needed helper: domain-local container directory for a file
fn container_dir(rel_path: &Path, folder: &str) -> PathBuf {
    let mut components = rel_path.components();
    let _file = components.next_back();
    match components.next() {
        Some(domain) => PathBuf::from(domain.as_os_str()).join(folder),
        None => PathBuf::from(folder),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{check, container_dir};
    use crate::checkers::container_placement::ContainerPlacement;
    use crate::common;
    use crate::Project;

    fn project(files: &[(&str, &str)]) -> Project {
        let mut project = Project {
            dunder_paths: common::default_dunder_paths(),
            ..Project::default()
        };
        for (path, source) in files {
            let file: syn::File = syn::parse_str(source).unwrap();
            project.parsed_files.insert(PathBuf::from(path), file);
        }
        project
    }

    #[test]
    fn test_usage() {
        let project = project(&[(
            "discovery/game.rs",
            "#[derive(Debug)] pub struct Game(pub String);\n",
        )]);
        let diags = check(&ContainerPlacement, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("newtypes.rs"));
        assert!(diags[0].message.contains("discovery/__basic__"));
    }

    #[test]
    fn test_usage_allows_container_item() {
        let project = project(&[(
            "discovery/__basic__/newtypes.rs",
            "#[derive(Debug, Clone)] pub struct Game(pub String);\n",
        )]);
        let diags = check(&ContainerPlacement, &project);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_flags_behavior_in_container() {
        let project = project(&[(
            "discovery/__basic__/newtypes.rs",
            "#[derive(Debug)] pub struct Game(pub String);\nimpl Game { pub fn name(&self) -> &str { \"\" } }\n",
        )]);
        let diags = check(&ContainerPlacement, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("has impls"));
    }

    #[test]
    fn test_usage_container_dir() {
        assert_eq!(
            container_dir(&PathBuf::from("discovery/session/room.rs"), "__basic__"),
            PathBuf::from("discovery/__basic__")
        );
        assert_eq!(
            container_dir(&PathBuf::from("lib_types.rs"), "__basic__"),
            PathBuf::from("__basic__")
        );
    }
}
