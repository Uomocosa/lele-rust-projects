use std::path::Path;

use super::no_stuttered_type::NoStutteredType;
use crate::common;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &NoStutteredType, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        if is_exempt_path(rel_path) {
            continue;
        }
        let Some(dir) = parent_dir_name(rel_path) else {
            continue;
        };
        let stem = rel_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let Some(name) = primary_exposed_type(file, stem) else {
            continue;
        };
        let Some(suffix) = strip_dir_prefix(&common::to_snake_case(&name), &dir) else {
            continue;
        };
        let suggested = common::to_pascal_case(&suffix);
        diags.push(Diagnostic {
            file: project.src_dir.join(rel_path),
            line: 1,
            col: 0,
            code: NoStutteredType::CODE.to_string(),
            message: format!(
                "type `{name}` repeats parent module `{dir}` — rename to `{suggested}` (`{dir}::{suggested}`)"
            ),
            severity: Severity::Error,
        });
    }

    diags
}

// needed helper: path exemption logic (mirrors atomic_file scope)
fn is_exempt_path(rel_path: &Path) -> bool {
    let file_name = rel_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name == "mod.rs" || file_name == "lib.rs" || file_name == "constants.rs" {
        return true;
    }
    rel_path
        .components()
        .any(|c| c.as_os_str().to_str() == Some("tests"))
}

// needed helper: parent directory name; crate-root files have no parent module
fn parent_dir_name(rel_path: &Path) -> Option<String> {
    rel_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(str::to_string)
}

// needed helper: first file-level exposed struct/enum whose snake name matches the stem
fn primary_exposed_type(file: &syn::File, stem: &str) -> Option<String> {
    file.items.iter().find_map(|item| {
        let (ident, vis) = match item {
            syn::Item::Struct(s) => (&s.ident, &s.vis),
            syn::Item::Enum(e) => (&e.ident, &e.vis),
            _ => return None,
        };
        if !is_exposed(vis) {
            return None;
        }
        let name = ident.to_string();
        if common::to_snake_case(&name) == stem {
            Some(name)
        } else {
            None
        }
    })
}

// needed helper: `pub` or `pub(crate)` visibility check
fn is_exposed(vis: &syn::Visibility) -> bool {
    match vis {
        syn::Visibility::Public(_) => true,
        syn::Visibility::Restricted(r) => {
            r.path.segments.len() == 1
                && r.path.segments.first().is_some_and(|s| s.ident == "crate")
        }
        syn::Visibility::Inherited => false,
    }
}

// needed helper: strict dir-prefix strip; exact matches and short suffixes are exempt
fn strip_dir_prefix(snake: &str, dir: &str) -> Option<String> {
    let rest = snake.strip_prefix(dir)?;
    let suffix = rest.strip_prefix('_')?;
    if suffix.is_empty() || suffix.len() < 3 {
        return None;
    }
    Some(suffix.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::strip_dir_prefix;
    use crate::Project;

    fn project_with(files: Vec<(&str, &str)>) -> Project {
        let parsed_files = files
            .into_iter()
            .map(|(path, code)| {
                (
                    PathBuf::from(path),
                    syn::parse_str(code).expect("bad fixture"),
                )
            })
            .collect::<HashMap<_, _>>();
        Project {
            root: PathBuf::from("."),
            src_dir: PathBuf::from("src"),
            entries: Vec::new(),
            module_info: HashMap::default(),
            parsed_files,
        }
    }

    #[test]
    fn test_usage() {
        let project = project_with(vec![(
            "freenet/freenet_client.rs",
            "pub struct FreenetClient(pub String);",
        )]);
        let diags = super::check(&super::NoStutteredType, &project);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "E027");
        assert!(diags[0].message.contains("`Client`"));
    }

    #[test]
    fn test_usage_exact_match_is_exempt() {
        let project = project_with(vec![
            ("cli/cli.rs", "pub struct Cli;"),
            ("roster/roster.rs", "pub struct Roster;"),
        ]);
        let diags = super::check(&super::NoStutteredType, &project);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_non_prefix_is_exempt() {
        let project = project_with(vec![
            ("p2p/event.rs", "pub enum Event<T> { A(T) }"),
            ("plugin/p2p_plugin.rs", "pub struct P2PPlugin;"),
            ("relay/letter_request.rs", "pub struct LetterRequest;"),
        ]);
        let diags = super::check(&super::NoStutteredType, &project);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_short_suffix_is_exempt() {
        let project = project_with(vec![(
            "net_id/network_id.rs",
            "pub struct NetworkId(pub u64);",
        )]);
        let diags = super::check(&super::NoStutteredType, &project);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_root_and_private_are_exempt() {
        let project = project_with(vec![
            ("freenet_client.rs", "pub struct FreenetClient;"),
            ("freenet/hidden.rs", "struct Hidden;"),
        ]);
        let diags = super::check(&super::NoStutteredType, &project);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_strip_dir_prefix() {
        assert_eq!(
            strip_dir_prefix("freenet_client", "freenet"),
            Some("client".to_string())
        );
        assert_eq!(strip_dir_prefix("p2p_plugin", "plugin"), None);
        assert_eq!(strip_dir_prefix("cli", "cli"), None);
        assert_eq!(strip_dir_prefix("network_id", "net_id"), None);
    }
}
