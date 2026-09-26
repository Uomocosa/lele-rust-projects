use syn::spanned::Spanned;

use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::Dunder;
use crate::Project;
use crate::Severity;

pub fn check(_self: &checkers::domain_import::DomainImport, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        for item in &file.items {
            if let syn::Item::Use(item_use) = item {
                if let Some(msg) = check_import(item_use, &project.dunder) {
                    diags.push(Diagnostic {
                        file: project.src_dir.join(rel_path),
                        line: find_use_line(item_use),
                        col: 0,
                        code: "E011".to_string(),
                        message: msg,
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    diags
}

// needed helper: import style validation
fn check_import(item_use: &syn::ItemUse, dunder: &Dunder) -> Option<String> {
    let segments = collect_use_segments(&item_use.tree);
    let first = segments.first()?;

    if first == "crate" && segments.len() >= 3 {
        if segments.iter().any(|segment| {
            dunder
                .folders
                .values()
                .any(|module| module.as_str() == segment)
        }) {
            return None;
        }
        if let [.., prev, last] = &segments[..] {
            if common::is_stuttered_path(prev, last) {
                return None;
            }
        }
        let direct = segments.join("::");
        let second = segments.get(1)?;
        if !is_pub_use(item_use) {
            return Some(format!(
                "use `use crate::{second};` instead of `use {direct};`"
            ));
        }
    }

    None
}

// needed helper: visibility check
fn is_pub_use(item_use: &syn::ItemUse) -> bool {
    matches!(item_use.vis, syn::Visibility::Public(_))
}

// needed helper: use tree segment collection
fn collect_use_segments(tree: &syn::UseTree) -> Vec<String> {
    match tree {
        syn::UseTree::Path(p) => {
            let mut segs = vec![p.ident.to_string()];
            segs.extend(collect_use_segments(&p.tree));
            segs
        }
        syn::UseTree::Name(n) => vec![n.ident.to_string()],
        syn::UseTree::Rename(r) => vec![r.ident.to_string()],
        syn::UseTree::Glob(_) => Vec::new(),
        syn::UseTree::Group(_) => Vec::new(),
    }
}

// needed helper: 1-based source line of a use statement via its span
fn find_use_line(item_use: &syn::ItemUse) -> usize {
    item_use.span().start().line
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{check, check_import, is_pub_use};
    use crate::checkers;
    use crate::checkers::domain_import::DomainImport;
    use crate::Dunder;
    use crate::Entry;
    use crate::EntryKind;
    use crate::Project;

    #[test]
    fn test_usage() {
        let checker = checkers::domain_import::DomainImport;
        let project = Project::default();
        assert!(check(&checker, &project).is_empty());
    }
    use syn::parse_quote;

    #[test]
    fn test_usage_flags_direct_type_import() {
        let u: syn::ItemUse = parse_quote! { use crate::player::PlayerId; };
        assert!(check_import(&u, &Dunder::default()).is_some());
    }

    #[test]
    fn test_usage_allows_domain_import() {
        let u: syn::ItemUse = parse_quote! { use crate::player; };
        assert!(check_import(&u, &Dunder::default()).is_none());
    }

    #[test]
    fn test_usage_flags_subfolder_import() {
        let u: syn::ItemUse = parse_quote! { use crate::clicker::plugin::ClickerPlugin; };
        assert!(check_import(&u, &Dunder::default()).is_some());
    }

    #[test]
    fn test_usage_allows_super_import() {
        let u: syn::ItemUse = parse_quote! { use super::player_new; };
        assert!(check_import(&u, &Dunder::default()).is_none());
    }

    #[test]
    fn test_usage_exempts_pub_use() {
        let u: syn::ItemUse = parse_quote! { pub use player::Player; };
        assert!(is_pub_use(&u));
    }

    #[test]
    fn test_usage_allows_direct_stutter_import() {
        let u: syn::ItemUse = parse_quote! { use crate::diagnostic::Diagnostic; };
        assert!(check_import(&u, &Dunder::default()).is_none());
    }

    #[test]
    fn test_usage_still_flags_direct_non_stutter_import() {
        let u: syn::ItemUse = parse_quote! { use crate::module_info::ModuleInfoMap; };
        assert!(check_import(&u, &Dunder::default()).is_some());
    }

    #[test]
    fn test_usage_reports_offending_line() {
        let dir = tempfile::tempdir().unwrap();
        let source = "use crate::player;\nuse crate::player::PlayerId;\n";
        let absolute_path = dir.path().join("probe.rs");
        std::fs::write(&absolute_path, source).unwrap();
        let mut project = Project {
            src_dir: dir.path().to_path_buf(),
            ..Project::default()
        };
        project.entries.push(Entry {
            relative_path: PathBuf::from("probe.rs"),
            absolute_path,
            kind: EntryKind::File,
        });
        project
            .parsed_files
            .insert(PathBuf::from("probe.rs"), syn::parse_str(source).unwrap());
        let diags = check(&DomainImport, &project);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].line, 2);
    }
}

// no test_usage necessary
