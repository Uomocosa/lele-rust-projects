// no test_usage necessary

use std::path::Path;

use crate::checker::Checker;
use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::project::Project;
use crate::severity::Severity;

use super::domain_import_register;
// needed helper: parsing utilities

pub struct DomainImport;

impl Checker for DomainImport {
    fn name(&self) -> &'static str {
        "domain_import"
    }

    fn code(&self) -> &'static str {
        "E011"
    }

    fn check(&self, project: &Project) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (rel_path, file) in &project.parsed_files {
            let is_struct_file = is_struct_delegate_file(file);

            for item in &file.items {
                if let syn::Item::Use(item_use) = item {
                    if let Some(msg) = check_import(item_use, is_struct_file) {
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
}

#[rustfmt::skip]
impl DomainImport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        domain_import_register::register(checkers, config)
    }
}

fn is_struct_delegate_file(file: &syn::File) -> bool {
    for item in &file.items {
        if let syn::Item::Impl(impl_block) = item {
            for attr in &impl_block.attrs {
                if attr.path().is_ident("rustfmt") {
                    return true;
                }
            }
        }
    }
    false
}

fn check_import(item_use: &syn::ItemUse, is_struct_file: bool) -> Option<String> {
    let segments = collect_use_segments(&item_use.tree);
    if segments.is_empty() {
        return None;
    }

    if segments[0] == "crate" && segments.len() >= 3 {
        let direct = segments.join("::");
        if !is_pub_use(item_use) {
            return Some(format!(
                "use `use crate::{};` instead of `use {};`",
                segments[1], direct
            ));
        }
    }

    if is_struct_file {
        return None;
    }

    for seg in &segments {
        if seg == "super" && segments.len() >= 2 {
            return None;
        }
    }

    None
}

fn is_pub_use(item_use: &syn::ItemUse) -> bool {
    matches!(item_use.vis, syn::Visibility::Public(_))
}

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

fn find_use_line(
    entries: &[crate::entry::Entry],
    rel_path: &Path,
    _item_use: &syn::ItemUse,
) -> Option<usize> {
    let entry = entries.iter().find(|e| e.relative_path == rel_path)?;
    let content = std::fs::read_to_string(&entry.absolute_path).ok()?;

    for (i, line) in content.lines().enumerate() {
        if line.trim().starts_with("use crate::") && line.contains(";") {
            return Some(i + 1);
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
        let u: syn::ItemUse = parse_quote! { use crate::player::Player; };
        assert!(check_import(&u, false).is_some());
    }

    #[test]
    fn test_usage_allows_domain_import() {
        let u: syn::ItemUse = parse_quote! { use crate::player; };
        assert!(check_import(&u, false).is_none());
    }

    #[test]
    fn test_usage_allows_super_in_struct_file() {
        let u: syn::ItemUse = parse_quote! { use super::player_new; };
        let result = check_import(&u, true);
        assert!(result.is_none());
    }

    #[test]
    fn test_usage_exempts_pub_use() {
        let u: syn::ItemUse = parse_quote! { pub use player::Player; };
        assert!(is_pub_use(&u));
    }
}
