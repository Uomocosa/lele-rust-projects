use std::path::Path;

use crate::checker::{Checker, Diagnostic, Severity};
use crate::config::Config;
use crate::project::Project;

use super::no_positional_register;
// needed helper: parsing utilities

pub struct NoPositional;

impl Checker for NoPositional {
    fn name(&self) -> &'static str {
        "no_positional"
    }

    fn code(&self) -> &'static str {
        "E009"
    }

    fn check(&self, project: &Project) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (rel_path, file) in &project.parsed_files {
            if !has_positional_types(file) {
                continue;
            }

            scan_block_for_positional(&file.items, rel_path, project, &mut diags);
        }

        diags
    }
}

#[rustfmt::skip]
impl NoPositional {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_positional_register::register(checkers, config)
    }
}

fn has_positional_types(file: &syn::File) -> bool {
    file.items.iter().any(|item| {
        if let syn::Item::Struct(s) = item {
            return s.fields.iter().any(|f| f.ident.is_none());
        }
        false
    })
}

fn scan_block_for_positional(
    items: &[syn::Item],
    rel_path: &Path,
    project: &Project,
    diags: &mut Vec<Diagnostic>,
) {
    for item in items {
        match item {
            syn::Item::Impl(impl_block) => {
                for impl_item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        scan_stmts(&method.block.stmts, rel_path, project, diags);
                    }
                }
            }
            syn::Item::Fn(func) => {
                scan_stmts(&func.block.stmts, rel_path, project, diags);
            }
            syn::Item::Mod(module) => {
                if let Some((_, inner)) = &module.content {
                    scan_block_for_positional(inner, rel_path, project, diags);
                }
            }
            _ => {}
        }
    }
}

fn scan_stmts(
    stmts: &[syn::Stmt],
    rel_path: &Path,
    project: &Project,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            syn::Stmt::Expr(expr, _) => scan_expr(expr, rel_path, project, diags),
            syn::Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    scan_expr(&init.expr, rel_path, project, diags);
                }
            }
            syn::Stmt::Macro(m) => {
                let content = m.mac.tokens.to_string();
                if has_positional_access(&content) {
                    diags.push(Diagnostic {
                        file: project.src_dir.join(rel_path),
                        line: 1,
                        col: 0,
                        code: "E009".to_string(),
                        message: "positional field access like `.0` or `.1` is not allowed — define the struct with named fields instead"
                            .to_string(),
                        severity: Severity::Error,
                    });
                }
            }
            _ => {}
        }
    }
}

fn scan_expr(expr: &syn::Expr, rel_path: &Path, project: &Project, diags: &mut Vec<Diagnostic>) {
    if let syn::Expr::Field(field) = expr {
        if matches!(&field.member, syn::Member::Unnamed(_)) {
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line: 1,
                col: 0,
                code: "E009".to_string(),
                message: "positional field access is not allowed, use named fields".to_string(),
                severity: Severity::Error,
            });
        }
    }
}

fn has_positional_access(content: &str) -> bool {
    content.contains(".0") || content.contains(".1")
}

#[cfg(test)]
mod tests {
    use super::has_positional_types;

    #[test]
    fn test_usage() {
        let file: syn::File = syn::parse_str(
            "pub struct Pos(pub String, pub u32);\npub struct Named { pub x: u32 }\n",
        )
        .unwrap();
        assert!(has_positional_types(&file));
    }
}
