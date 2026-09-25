use derive_more::{Deref, DerefMut};
use syn::spanned::Spanned;
use syn::visit::Visit;

use super::no_super_imports::NoSuperImports;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &NoSuperImports, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        let mut hits = Vec::new();
        {
            let mut visitor = SuperPathVisitor(&mut hits);
            visitor.visit_file(file);
        }
        for hit in hits {
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line: hit.line,
                col: 0,
                code: "E033".to_string(),
                message: format!(
                    "`{}` used outside `#[cfg(test)]` — add `use crate::<domain>;` at the top of the file and reference `<domain>::…` instead",
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
    use super::SuperPathVisitor;
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
}

// no test_usage necessary
