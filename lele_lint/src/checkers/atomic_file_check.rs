use std::path::Path;

use super::atomic_file::AtomicFile;
use crate::common;
use crate::diagnostic;
use crate::project;
use crate::severity;

pub(crate) fn check(_self: &AtomicFile, project: &project::Project) -> Vec<diagnostic::Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        let file_name = rel_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if is_exempt_path(rel_path, file_name) {
            continue;
        }

        let pub_items = collect_pub_items(file);
        if pub_items.is_empty() {
            continue;
        }

        let file_stem = rel_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let primary = &pub_items[0];

        check_filename_match(&primary.name, file_stem, rel_path, project, &mut diags);

        for extra in &pub_items[1..] {
            let suggested_file = format!("{}_{}.rs", file_stem, extra.name);
            diags.push(diagnostic::Diagnostic {
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
                severity: severity::Severity::Error,
            });
        }
    }

    diags
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
    project: &project::Project,
    diags: &mut Vec<diagnostic::Diagnostic>,
) {
    let expected = common::to_snake_case(name);

    if expected == file_stem {
        return;
    }

    if file_stem.contains('_') {
        if let Some((_prefix, suffix)) = file_stem.rsplit_once('_') {
            if expected.ends_with(suffix) {
                return;
            }
        }
    }

    diags.push(diagnostic::Diagnostic {
        file: project.src_dir.join(rel_path),
        line: 1,
        col: 0,
        code: "E001".to_string(),
        message: format!("filename mismatch — `{file_stem}.rs` should be `{expected}.rs`"),
        severity: severity::Severity::Error,
    });
}

#[cfg(test)]
mod tests {
    use super::{collect_pub_items, is_exempt_path};
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
}
