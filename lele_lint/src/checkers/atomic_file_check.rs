use std::collections::HashSet;
use std::path::Path;

use super::atomic_file::AtomicFile;
use crate::common;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &AtomicFile, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let known_stems = known_type_stems(project);

    for (rel_path, file) in &project.parsed_files {
        let file_name = rel_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if is_exempt_path(rel_path, file_name) {
            continue;
        }

        let pub_items = collect_pub_items(file);
        let Some(primary) = pub_items.first() else {
            continue;
        };

        let file_stem = rel_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        check_filename_match(
            &primary.name,
            file_stem,
            rel_path,
            project,
            &known_stems,
            &mut diags,
        );

        let rest = pub_items.get(1..).unwrap_or(&[]);
        for extra in rest {
            let suggested_file = format!("{}_{}.rs", file_stem, extra.name);
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line: 1,
                col: 0,
                code: "E001".to_string(),
                message: format!(
                    "only one public item per file — move `pub {} {}` to `{}`",
                    extra.kind.kind_str(),
                    extra.name,
                    suggested_file
                ),
                severity: Severity::Error,
            });
        }

        check_fn_file_purity(file, file_stem, rel_path, project, &mut diags);
    }

    diags
}

// needed helper: SHAPE-F fn-file purity — a file whose primary item is a fn
// may hold private helpers but no exposed (`pub`/`pub(crate)`) types or consts
fn check_fn_file_purity(
    file: &syn::File,
    file_stem: &str,
    rel_path: &Path,
    project: &Project,
    diags: &mut Vec<Diagnostic>,
) {
    let has_pub_fn = file
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Fn(f) if is_exposed(&f.vis)));
    if !has_pub_fn {
        return;
    }
    for item in &file.items {
        let (name, kind) = match item {
            syn::Item::Struct(s) if is_exposed(&s.vis) => (s.ident.to_string(), "struct"),
            syn::Item::Enum(e) if is_exposed(&e.vis) => (e.ident.to_string(), "enum"),
            syn::Item::Const(c) if is_exposed(&c.vis) => (c.ident.to_string(), "const"),
            syn::Item::Static(s) if is_exposed(&s.vis) => (s.ident.to_string(), "static"),
            syn::Item::Type(t) if is_exposed(&t.vis) => (t.ident.to_string(), "type"),
            syn::Item::Trait(t) if is_exposed(&t.vis) => (t.ident.to_string(), "trait"),
            syn::Item::Union(u) if is_exposed(&u.vis) => (u.ident.to_string(), "union"),
            _ => continue,
        };
        let home = if kind == "const" || kind == "static" {
            "nearest `constants.rs`"
        } else {
            &format!("`{}.rs`", common::to_snake_case(&name))
        };
        diags.push(Diagnostic {
            file: project.src_dir.join(rel_path),
            line: 1,
            col: 0,
            code: "E001".to_string(),
            message: format!(
                "SHAPE-F fn-file `{file_stem}.rs` must hold only the fn — move exposed `{kind} {name}` to {home} (O2-extraction)"
            ),
            severity: Severity::Error,
        });
    }
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

struct PubItem {
    name: String,
    kind: PubItemKind,
}

enum PubItemKind {
    Struct,
    Enum,
    Fn,
}

impl PubItemKind {
    fn kind_str(&self) -> &'static str {
        match self {
            PubItemKind::Struct => "struct",
            PubItemKind::Enum => "enum",
            PubItemKind::Fn => "fn",
        }
    }
}

// needed helper: path exemption logic
fn is_exempt_path(rel_path: &Path, file_name: &str) -> bool {
    if file_name == "mod.rs" || file_name == "lib.rs" || file_name == "constants.rs" {
        return true;
    }
    rel_path
        .components()
        .any(|c| c.as_os_str().to_str() == Some("tests"))
}

// needed helper: pub item collection
fn collect_pub_items(file: &syn::File) -> Vec<PubItem> {
    let mut items = Vec::new();
    for item in &file.items {
        match item {
            syn::Item::Struct(s) if matches!(s.vis, syn::Visibility::Public(_)) => {
                items.push(PubItem {
                    name: s.ident.to_string(),
                    kind: PubItemKind::Struct,
                });
            }
            syn::Item::Enum(e) if matches!(e.vis, syn::Visibility::Public(_)) => {
                items.push(PubItem {
                    name: e.ident.to_string(),
                    kind: PubItemKind::Enum,
                });
            }
            syn::Item::Fn(f) if matches!(f.vis, syn::Visibility::Public(_)) => {
                items.push(PubItem {
                    name: f.sig.ident.to_string(),
                    kind: PubItemKind::Fn,
                });
            }
            _ => {}
        }
    }
    items
}

// needed helper: filename validation against snake_case
fn check_filename_match(
    name: &str,
    file_stem: &str,
    rel_path: &Path,
    project: &Project,
    known_stems: &HashSet<String>,
    diags: &mut Vec<Diagnostic>,
) {
    let expected = common::to_snake_case(name);

    if expected == file_stem {
        return;
    }

    if file_stem.contains('_') && has_known_parent_prefix(file_stem, known_stems) {
        if let Some((_prefix, suffix)) = file_stem.rsplit_once('_') {
            if expected.ends_with(suffix) {
                return;
            }
        }
    }

    diags.push(Diagnostic {
        file: project.src_dir.join(rel_path),
        line: 1,
        col: 0,
        code: "E001".to_string(),
        message: format!("filename mismatch — `{file_stem}.rs` should be `{expected}.rs`"),
        severity: Severity::Error,
    });
}

// needed helper: known primary-type file stems in the project
fn known_type_stems(project: &Project) -> HashSet<String> {
    project
        .parsed_files
        .iter()
        .filter_map(|(rel_path, file)| {
            let stem = rel_path.file_stem()?.to_str()?;
            common::primary_type_name(file, stem)?;
            Some(stem.to_string())
        })
        .collect()
}

// needed helper: any underscore-prefix of the stem names a known type
fn has_known_parent_prefix(file_stem: &str, known_stems: &HashSet<String>) -> bool {
    file_stem.match_indices('_').any(|(pos, _)| {
        file_stem
            .get(..pos)
            .is_some_and(|prefix| known_stems.contains(prefix))
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        check_filename_match, check_fn_file_purity, collect_pub_items, has_known_parent_prefix,
        is_exempt_path, is_exposed,
    };
    use crate::Project;
    use std::path::Path;

    #[test]
    fn test_usage() {
        let file: syn::File =
            syn::parse_str("pub struct AtomicFile;\npub fn helper() {}\nfn private() {}").unwrap();
        let items = collect_pub_items(&file);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "AtomicFile");
        assert_eq!(items[1].name, "helper");

        assert!(is_exempt_path(Path::new("checkers/mod.rs"), "mod.rs"));
        assert!(!is_exempt_path(
            Path::new("checkers/atomic_file.rs"),
            "atomic_file.rs"
        ));
    }

    #[test]
    fn test_usage_suffix_exemption_needs_known_parent() {
        let project = Project {
            root: std::path::PathBuf::from("."),
            src_dir: std::path::PathBuf::from("src"),
            entries: Vec::new(),
            module_info: std::collections::HashMap::default(),
            parsed_files: std::collections::HashMap::default(),
        };
        let known: HashSet<String> = HashSet::from(["lele_config".to_string()]);
        let empty: HashSet<String> = HashSet::new();

        let mut diags = Vec::new();
        check_filename_match(
            "load",
            "lele_config_load",
            Path::new("lele_config_load.rs"),
            &project,
            &known,
            &mut diags,
        );
        assert!(diags.is_empty());

        let mut diags = Vec::new();
        check_filename_match(
            "load",
            "config_load",
            Path::new("config_load.rs"),
            &project,
            &empty,
            &mut diags,
        );
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("filename mismatch"));

        assert!(!has_known_parent_prefix("freenet_client_connect", &known));
        let with_prefix: HashSet<String> = HashSet::from(["freenet_client".to_string()]);
        assert!(has_known_parent_prefix(
            "freenet_client_connect",
            &with_prefix
        ));
    }

    #[test]
    fn test_usage_fn_file_purity_flags_exposed_type() {
        let project = Project {
            root: std::path::PathBuf::from("."),
            src_dir: std::path::PathBuf::from("src"),
            entries: Vec::new(),
            module_info: std::collections::HashMap::default(),
            parsed_files: std::collections::HashMap::default(),
        };
        let file: syn::File =
            syn::parse_str("pub struct Cache { pub x: u32 }\npub fn discover() {}\n").unwrap();
        let mut diags = Vec::new();
        check_fn_file_purity(
            &file,
            "discover",
            Path::new("discover.rs"),
            &project,
            &mut diags,
        );
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("SHAPE-F"));

        let vis_pub: syn::Visibility = syn::parse_quote! { pub };
        let vis_crate: syn::Visibility = syn::parse_quote! { pub(crate) };
        let vis_priv: syn::Visibility = syn::parse_quote! {};
        assert!(is_exposed(&vis_pub));
        assert!(is_exposed(&vis_crate));
        assert!(!is_exposed(&vis_priv));
    }

    #[test]
    fn test_usage_fn_file_purity_allows_private_struct() {
        let project = Project {
            root: std::path::PathBuf::from("."),
            src_dir: std::path::PathBuf::from("src"),
            entries: Vec::new(),
            module_info: std::collections::HashMap::default(),
            parsed_files: std::collections::HashMap::default(),
        };
        let file: syn::File =
            syn::parse_str("struct Cache { x: u32 }\npub fn discover() {}\n").unwrap();
        let mut diags = Vec::new();
        check_fn_file_purity(
            &file,
            "discover",
            Path::new("discover.rs"),
            &project,
            &mut diags,
        );
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_fn_file_purity_flags_pub_const() {
        let project = Project {
            root: std::path::PathBuf::from("."),
            src_dir: std::path::PathBuf::from("src"),
            entries: Vec::new(),
            module_info: std::collections::HashMap::default(),
            parsed_files: std::collections::HashMap::default(),
        };
        let file: syn::File =
            syn::parse_str("pub const MAX: usize = 8;\npub fn discover() {}\n").unwrap();
        let mut diags = Vec::new();
        check_fn_file_purity(
            &file,
            "discover",
            Path::new("discover.rs"),
            &project,
            &mut diags,
        );
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("constants.rs"));
    }
}
