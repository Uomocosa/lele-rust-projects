use std::path::Path;

use derive_more::{Deref, DerefMut};
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::checkers;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub fn check(
    _self: &checkers::no_super_imports::NoSuperImports,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        let mut hits = Vec::new();
        {
            let mut visitor = SuperPathVisitor(&mut hits);
            visitor.visit_file(file);
        }
        for hit in hits {
            let (import, replacement) = suggestion(rel_path, &hit.path);
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line: hit.line,
                col: 0,
                code: "E033".to_string(),
                message: format!(
                    "`{}` used outside `#[cfg(test)]` — write `{replacement}` instead and add `{import}` at the top of the file",
                    hit.path
                ),
                severity: Severity::Error,
            });
        }
    }

    diags
}

// needed helper: collected super path hit data
struct SuperPathHit {
    line: usize,
    path: String,
}

#[derive(Deref, DerefMut)]
struct SuperPathVisitor<'a>(&'a mut Vec<SuperPathHit>);

impl<'ast> Visit<'ast> for SuperPathVisitor<'_> {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if has_cfg_test(&item.attrs) {
            return;
        }
        if starts_with_super(&item.tree) {
            self.push(SuperPathHit {
                line: item.span().start().line,
                path: use_tree_string(&item.tree),
            });
        }
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path
            .segments
            .first()
            .is_some_and(|seg| seg.ident == "super")
        {
            let path_str = path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            self.push(SuperPathHit {
                line: path.span().start().line,
                path: path_str,
            });
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if has_cfg_test(&item.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if has_cfg_test(&item.attrs) {
            return;
        }
        syn::visit::visit_item_fn(self, item);
    }
}

// needed helper: concrete import + replacement for a super-path hit
fn suggestion(rel_path: &Path, hit: &str) -> (String, String) {
    let Some(rest) = hit.strip_prefix("super::") else {
        return generic_suggestion();
    };
    if rest.starts_with("super::") {
        return generic_suggestion();
    }
    let mut parts = rel_path.components();
    let first = parts.next().and_then(|c| c.as_os_str().to_str());
    match first {
        Some(domain) if parts.next().is_some() => {
            (format!("use crate::{domain};"), format!("{domain}::{rest}"))
        }
        _ => {
            let head = rest.split("::").next().unwrap_or(rest);
            (format!("use crate::{head};"), rest.to_string())
        }
    }
}

// needed helper: fallback suggestion when no concrete fix applies
fn generic_suggestion() -> (String, String) {
    (
        "use crate::<domain>;".to_string(),
        "<domain>::…".to_string(),
    )
}
// needed helper: cfg(test) attribute detection on any item
fn has_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && matches!(&attr.meta, syn::Meta::List(list) if list.tokens.to_string().contains("test"))
    })
}

// needed helper: leading `super` use-tree detection
fn starts_with_super(tree: &syn::UseTree) -> bool {
    matches!(tree, syn::UseTree::Path(path) if path.ident == "super")
}

// needed helper: use-tree rendering for diagnostics
fn use_tree_string(tree: &syn::UseTree) -> String {
    match tree {
        syn::UseTree::Path(path) => format!("{}::{}", path.ident, use_tree_string(&path.tree)),
        syn::UseTree::Name(name) => name.ident.to_string(),
        syn::UseTree::Rename(rename) => format!("{} as {}", rename.ident, rename.rename),
        syn::UseTree::Glob(_) => "*".to_string(),
        syn::UseTree::Group(_) => "{...}".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{check, SuperPathVisitor};
    use crate::checkers::no_super_imports::NoSuperImports;
    use crate::Project;
    use syn::visit::Visit;

    fn hit_lines(src: &str) -> Vec<usize> {
        let file = syn::parse_file(src).unwrap();
        let mut hits = Vec::new();
        {
            let mut visitor = SuperPathVisitor(&mut hits);
            visitor.visit_file(&file);
        }
        hits.into_iter().map(|hit| hit.line).collect()
    }

    #[test]
    fn test_usage() {
        assert_eq!(hit_lines("use super::config_load;\n"), vec![1]);
        assert_eq!(hit_lines("use super::super::Event;\n"), vec![1]);
        assert_eq!(hit_lines("fn f() { super::a::b(); }\n"), vec![1]);
    }

    #[test]
    fn test_usage_allows_super_in_cfg_test_mod() {
        assert_eq!(
            hit_lines("#[cfg(test)]\nmod tests { use super::drain_events; }\n"),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn test_usage_allows_super_in_cfg_test_fn() {
        assert_eq!(
            hit_lines("#[cfg(test)]\nfn helper() { super::a::b(); }\n"),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn test_usage_message_names_domain_fix() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("checkers/probe.rs"),
            syn::parse_str("use super::probe_check;\n").unwrap(),
        );
        let diags = check(&NoSuperImports, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("use crate::checkers;"));
        assert!(diags[0].message.contains("checkers::probe_check"));
    }

    #[test]
    fn test_usage_message_names_root_fix() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("probe.rs"),
            syn::parse_str("use super::sibling;\n").unwrap(),
        );
        let diags = check(&NoSuperImports, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("use crate::sibling;"));
    }
}

// no test_usage necessary
