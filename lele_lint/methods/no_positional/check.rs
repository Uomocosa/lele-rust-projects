use std::path::PathBuf;

use syn::visit::Visit;

use crate::checkers;
use crate::Diagnostic;
use crate::Project;

pub fn check(_self: &checkers::no_positional::NoPositional, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for source in project.content_sources() {
        let file_path = project.absolute_path(source.origin, source.relative_path);
        let mut visitor = PositionalVisitor {
            file_path,
            diags: &mut diags,
        };
        visitor.visit_file(source.file);
    }
    diags
}

// needed helper: reports every positional field access in the file
struct PositionalVisitor<'a> {
    file_path: PathBuf,
    diags: &'a mut Vec<Diagnostic>,
}

impl<'ast> Visit<'ast> for PositionalVisitor<'_> {
    fn visit_expr_field(&mut self, node: &'ast syn::ExprField) {
        if matches!(node.member, syn::Member::Unnamed(_)) {
            self.diags.push(Diagnostic {
                file: self.file_path.clone(),
                line: 1,
                col: 0,
                code: checkers::no_positional::NoPositional::CODE.to_string(),
                message: "positional field access is not allowed, use named fields".to_string(),
            });
        }
        syn::visit::visit_expr_field(self, node);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use syn::visit::Visit;

    use super::PositionalVisitor;
    use crate::Diagnostic;

    fn diags_for(source: &str) -> Vec<Diagnostic> {
        let file: syn::File = syn::parse_str(source).unwrap();
        let mut diags = Vec::new();
        let mut visitor = PositionalVisitor {
            file_path: PathBuf::from("probe.rs"),
            diags: &mut diags,
        };
        visitor.visit_file(&file);
        diags
    }

    #[test]
    fn test_usage() {
        let diags = diags_for(
            "pub struct Pos(pub String, pub u32);\npub fn f(p: &Pos) -> &String { &p.0 }\n",
        );
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_usage_catches_nested_access() {
        let diags = diags_for(
            "pub struct Pos(pub String);\npub fn f(p: &Pos) -> String { g(p.0, h().1) }\n",
        );
        assert_eq!(diags.len(), 2);
    }

    #[test]
    fn test_usage_ignores_floats_in_macros() {
        let diags = diags_for("pub fn f() -> f64 { let x = 0.0; println!(\"{x}\"); x }\n");
        assert!(diags.is_empty());
    }
}
