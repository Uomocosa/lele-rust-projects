use std::path::Path;

use super::domain_import::DomainImport;
use crate::common;
use crate::Diagnostic;
use crate::Entry;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &DomainImport, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        for item in &file.items {
            if let syn::Item::Use(item_use) = item {
                if let Some(msg) = check_import(item_use) {
                    if let Some(line) = find_use_line(&project.entries, rel_path, item_use) {
                        diags.push(Diagnostic {
                            file: project.src_dir.join(rel_path),
                            line,
                            col: 0,
                            code: "E011".to_string(),
                            message: msg,
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }
    }

    diags
}

// needed helper: import style validation
fn check_import(item_use: &syn::ItemUse) -> Option<String> {
    let segments = collect_use_segments(&item_use.tree);
    let first = segments.first()?;

    if first == "crate" && segments.len() >= 3 {
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

// needed helper: source line lookup for use statements
// Known limitation: reports the first `use crate::` line in the file, not the line
// of the specific import — diagnostics on multi-import files point at the wrong line.
fn find_use_line(entries: &[Entry], rel_path: &Path, _item_use: &syn::ItemUse) -> Option<usize> {
    let entry = entries.iter().find(|e| e.relative_path == rel_path)?;
    let content = std::fs::read_to_string(&entry.absolute_path).ok()?;

    for (i, line) in content.lines().enumerate() {
        if line.trim().starts_with("use crate::") && line.contains(";") {
            return Some(i.saturating_add(1));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{check_import, is_pub_use};
    use syn::parse_quote;

    #[test]
    fn test_usage_flags_direct_type_import() {
        let u: syn::ItemUse = parse_quote! { use crate::player::PlayerId; };
        assert!(check_import(&u).is_some());
    }

    #[test]
    fn test_usage_allows_domain_import() {
        let u: syn::ItemUse = parse_quote! { use crate::player; };
        assert!(check_import(&u).is_none());
    }

    #[test]
    fn test_usage_flags_subfolder_import() {
        let u: syn::ItemUse = parse_quote! { use crate::clicker::plugin::ClickerPlugin; };
        assert!(check_import(&u).is_some());
    }

    #[test]
    fn test_usage_allows_super_import() {
        let u: syn::ItemUse = parse_quote! { use super::player_new; };
        assert!(check_import(&u).is_none());
    }

    #[test]
    fn test_usage_exempts_pub_use() {
        let u: syn::ItemUse = parse_quote! { pub use player::Player; };
        assert!(is_pub_use(&u));
    }

    #[test]
    fn test_usage_allows_direct_stutter_import() {
        let u: syn::ItemUse = parse_quote! { use crate::diagnostic::Diagnostic; };
        assert!(check_import(&u).is_none());
    }

    #[test]
    fn test_usage_still_flags_direct_non_stutter_import() {
        let u: syn::ItemUse = parse_quote! { use crate::module_info::ModuleInfoMap; };
        assert!(check_import(&u).is_some());
    }
}

// no test_usage necessary
