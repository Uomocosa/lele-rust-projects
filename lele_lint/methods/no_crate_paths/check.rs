use derive_more::{Deref, DerefMut};
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::Project;

pub fn check(_self: &checkers::no_crate_paths::NoCratePaths, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for source in project.content_sources() {
        let rel_path = source.relative_path;
        let file = source.file;
        if common::is_crate_root(rel_path) {
            continue;
        }
        let mut hits = Vec::new();
        {
            let mut visitor = CratePathVisitor(&mut hits);
            visitor.visit_file(file);
        }
        for hit in std::mem::take(&mut hits) {
            diags.push(Diagnostic {
                file: project.absolute_path(source.origin, rel_path),
                line: hit.line,
                col: 0,
                code: checkers::no_crate_paths::NoCratePaths::CODE.to_string(),
                message: format!(
                    "`{}` path used outside a top-level `use` declaration — add `use crate::<module>;` at the top of the file and reference `<module>::…` instead",
                    hit.path
                ),
            });
        }
    }

    diags
}

// needed helper: AST visitor collecting `crate::` paths outside `use` items
struct CratePathHit {
    line: usize,
    path: String,
}

#[derive(Deref, DerefMut)]
struct CratePathVisitor<'a>(&'a mut Vec<CratePathHit>);

impl<'ast> Visit<'ast> for CratePathVisitor<'_> {
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
            self.push(CratePathHit {
                line: path.span().start().line,
                path: path_str,
            });
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
    use super::{check, CratePathVisitor};
    use crate::checkers;
    use crate::Project;
    use syn::visit::Visit;

    #[test]
    fn test_usage() {
        let checker = checkers::no_crate_paths::NoCratePaths;
        let project = Project::default();
        assert!(check(&checker, &project).is_empty());
    }

    fn hit_lines(src: &str) -> Vec<usize> {
        let file = syn::parse_file(src).unwrap();
        let mut hits = Vec::new();
        {
            let mut visitor = CratePathVisitor(&mut hits);
            visitor.visit_file(&file);
        }
        std::mem::take(&mut hits)
            .into_iter()
            .map(|hit| hit.line)
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
        assert_eq!(hit_lines(src), Vec::<usize>::new());
    }

    #[test]
    fn test_usage_allows_pub_crate_visibility() {
        let src = "pub(crate) mod boxes;\npub(crate) use boxes::PlayerId;\n";
        assert_eq!(hit_lines(src), Vec::<usize>::new());
    }

    #[test]
    fn test_usage_allows_super_and_self() {
        let src = "use super::player;\nfn f() { super::player::Player::default(); }\n";
        assert_eq!(hit_lines(src), Vec::<usize>::new());
    }
}
