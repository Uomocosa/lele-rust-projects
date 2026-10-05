use std::path::Path;

use crate::checkers;
use crate::Diagnostic;
use crate::Project;

pub fn check(_self: &checkers::no_positional::NoPositional, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for source in project.content_sources() {
        if !has_positional_types(source.file) {
            continue;
        }
        let file_path = project.absolute_path(source.origin, source.relative_path);
        scan_block_for_positional(&source.file.items, &file_path, &mut diags);
    }

    diags
}

// needed helper: positional type presence check
fn has_positional_types(file: &syn::File) -> bool {
    file.items.iter().any(|item| {
        if let syn::Item::Struct(s) = item {
            return s.fields.iter().any(|f| f.ident.is_none());
        }
        false
    })
}

// needed helper: recursive item block scanner
fn scan_block_for_positional(items: &[syn::Item], file_path: &Path, diags: &mut Vec<Diagnostic>) {
    for item in items {
        match item {
            syn::Item::Impl(impl_block) => {
                for impl_item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        scan_stmts(&method.block.stmts, file_path, diags);
                    }
                }
            }
            syn::Item::Fn(func) => {
                scan_stmts(&func.block.stmts, file_path, diags);
            }
            syn::Item::Mod(module) => {
                if let Some((_, inner)) = &module.content {
                    scan_block_for_positional(inner, file_path, diags);
                }
            }
            _ => {}
        }
    }
}

// needed helper: statement-level scanner
fn scan_stmts(stmts: &[syn::Stmt], file_path: &Path, diags: &mut Vec<Diagnostic>) {
    for stmt in stmts {
        match stmt {
            syn::Stmt::Expr(expr, _) => scan_expr(expr, file_path, diags),
            syn::Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    scan_expr(&init.expr, file_path, diags);
                }
            }
            syn::Stmt::Macro(m) => {
                let content = m.mac.tokens.to_string();
                if has_positional_access(&content) {
                    diags.push(Diagnostic {
                        file: file_path.to_path_buf(),
                        line: 1,
                        col: 0,
                        code: "E009".to_string(),
                        message: "positional field access like `.0` or `.1` is not allowed — define the struct with named fields instead"
                            .to_string(),
                    });
                }
            }
            _ => {}
        }
    }
}

// needed helper: expression-level position access checker
fn scan_expr(expr: &syn::Expr, file_path: &Path, diags: &mut Vec<Diagnostic>) {
    if let syn::Expr::Field(field) = expr {
        if matches!(&field.member, syn::Member::Unnamed(_)) {
            diags.push(Diagnostic {
                file: file_path.to_path_buf(),
                line: 1,
                col: 0,
                code: "E009".to_string(),
                message: "positional field access is not allowed, use named fields".to_string(),
            });
        }
    }
}

// needed helper: macro token string scan for .0/.1 access
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
