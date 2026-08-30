use std::path::Path;

use derive_more::{Deref, DerefMut};
use syn::spanned::Spanned;
use syn::visit::Visit;

use super::no_crate_paths::NoCratePaths;
use crate::diagnostic;
use crate::project;
use crate::severity;

pub(crate) fn check(
    _self: &NoCratePaths,
    project: &project::Project,
) -> Vec<diagnostic::Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        if is_crate_root(rel_path) {
            continue;
        }
        let mut visitor = CratePathVisitor::default();
        visitor.visit_file(file);
        for (line, path_str) in std::mem::take(&mut *visitor) {
            diags.push(diagnostic::Diagnostic {
                file: project.src_dir.join(rel_path),
                line,
                col: 0,
                code: "E020".to_string(),
                message: format!(
                    "`{path_str}` path used outside a top-level `use` declaration — add `use crate::<module>;` at the top of the file or use a `super::`-relative path instead"
                ),
                severity: severity::Severity::Error,
            });
        }
    }

    diags
}

// needed helper: crate root file detection
fn is_crate_root(rel_path: &Path) -> bool {
    matches!(
        rel_path.file_name().and_then(|n| n.to_str()),
        Some("lib.rs") | Some("main.rs")
    )
}

// needed helper: AST visitor collecting `crate::` paths outside `use` items
#[derive(Default, Deref, DerefMut)]
struct CratePathVisitor(Vec<(usize, String)>);

impl<'ast> Visit<'ast> for CratePathVisitor {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path
            .segments
            .first()
            .is_some_and(|seg| seg.ident == "crate")
        {
            let path_str = path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            self.push((path.span().start().line, path_str));
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_visibility(&mut self, visibility: &'ast syn::Visibility) {
        match visibility {
            syn::Visibility::Restricted(_) => {}
            _ => syn::visit::visit_visibility(self, visibility),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CratePathVisitor;
    use syn::visit::Visit;

    fn hit_lines(src: &str) -> Vec<usize> {
        let file = syn::parse_file(src).unwrap();
        let mut visitor = CratePathVisitor::default();
        visitor.visit_file(&file);
        std::mem::take(&mut *visitor)
            .into_iter()
            .map(|(line, _)| line)
            .collect()
    }

    #[test]
    fn test_usage_flags_inline_crate_path() {
        let src = r"
            use crate::boxes;
            pub fn connect(own_id: crate::boxes::PlayerId) {}
        ";
        assert_eq!(hit_lines(src), vec![3]);
    }

    #[test]
    fn test_usage_allows_top_level_use() {
        let src = "use crate::boxes;\nfn f() -> boxes::PlayerId { boxes::PlayerId::default() }\n";
        assert!(hit_lines(src).is_empty());
    }

    #[test]
    fn test_usage_allows_pub_crate_visibility() {
        let src = "pub(crate) mod boxes;\npub(crate) use boxes::PlayerId;\n";
        assert!(hit_lines(src).is_empty());
    }

    #[test]
    fn test_usage_allows_super_and_self() {
        let src = "use super::player;\nfn f() { super::player::Player::default(); }\n";
        assert!(hit_lines(src).is_empty());
    }
}
