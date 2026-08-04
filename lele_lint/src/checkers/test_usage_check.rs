// no test_usage necessary

use std::path::Path;

use super::test_usage::TestUsage;
use crate::diagnostic::Diagnostic;
use crate::entry_kind::EntryKind;
use crate::project::Project;
use crate::severity::Severity;

// needed helper: parsing utilities

pub(crate) fn check(_self: &TestUsage, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        if is_exempt(rel_path, file) {
            continue;
        }

        if has_test_usage_opt_out(project, rel_path) {
            continue;
        }

        if !has_test_usage(file) {
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line: 1,
                col: 0,
                code: "E006".to_string(),
                message: format!(
                    "file `{}` must contain a `#[cfg(test)] mod tests {{ fn test_usage() {{ ... }} }}` block, or add `// no test_usage necessary` to opt out",
                    rel_path.display()
                ),
                severity: Severity::Error,
            });
        }
    }

    diags
}

fn has_test_usage_opt_out(project: &Project, rel_path: &Path) -> bool {
    let entry = match project
        .entries
        .iter()
        .find(|e| e.relative_path == rel_path && e.kind == EntryKind::File)
    {
        Some(e) => e,
        None => return false,
    };

    let content = match std::fs::read_to_string(&entry.absolute_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    content
        .lines()
        .any(|line| line.trim().starts_with("// no test_usage necessary"))
}

fn is_exempt(rel_path: &Path, file: &syn::File) -> bool {
    let file_name = rel_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if (file_name == "mod.rs" || file_name == "lib.rs") && is_pure_module_tree(file) {
        return true;
    }

    if file_name == "constants.rs" {
        return true;
    }

    if rel_path
        .components()
        .any(|c| c.as_os_str().to_str() == Some("tests"))
    {
        return true;
    }

    is_type_only(file) || is_thin_delegate_only(file)
}

fn is_pure_module_tree(file: &syn::File) -> bool {
    if file.items.is_empty() {
        return true;
    }
    file.items.iter().all(|item| {
        matches!(
            item,
            syn::Item::Mod(_) | syn::Item::Use(_) | syn::Item::Macro(_)
        )
    })
}

fn is_type_only(file: &syn::File) -> bool {
    let has_struct_or_enum = file
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Struct(_) | syn::Item::Enum(_)));
    if !has_struct_or_enum {
        return false;
    }

    let has_non_default_impl = file.items.iter().any(|item| {
        if let syn::Item::Impl(impl_block) = item {
            if impl_block.trait_.is_some() {
                return false;
            }
            if !is_default_only_impl(impl_block) {
                return true;
            }
        }
        false
    });

    !has_non_default_impl
}

fn is_default_only_impl(impl_block: &syn::ItemImpl) -> bool {
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            if method.sig.ident != "default" {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

fn is_thin_delegate_only(file: &syn::File) -> bool {
    let mut has_default = false;
    let mut delegate_impls = false;

    for item in &file.items {
        if let syn::Item::Impl(impl_block) = item {
            if impl_block.trait_.is_some() {
                continue;
            }
            if is_default_only_impl(impl_block) {
                has_default = true;
            } else if is_likely_delegate_impl(impl_block) {
                delegate_impls = true;
            } else {
                return false;
            }
        }
    }

    has_default && delegate_impls
}

fn is_likely_delegate_impl(impl_block: &syn::ItemImpl) -> bool {
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            if !is_single_call_body(&method.block) {
                return false;
            }
        }
    }
    !impl_block.items.is_empty()
}

fn is_single_call_body(block: &syn::Block) -> bool {
    if block.stmts.len() != 1 {
        return false;
    }
    matches!(&block.stmts[0], syn::Stmt::Expr(_, _))
}

fn has_test_usage(file: &syn::File) -> bool {
    for item in &file.items {
        if let syn::Item::Mod(module) = item {
            if is_cfg_test(module) {
                if let Some((_, items)) = &module.content {
                    for inner in items {
                        if let syn::Item::Fn(func) = inner {
                            if func.sig.ident == "test_usage" {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn is_cfg_test(module: &syn::ItemMod) -> bool {
    module.attrs.iter().any(|attr| {
        if attr.path().is_ident("cfg") {
            if let syn::Meta::List(list) = &attr.meta {
                return list.tokens.to_string().contains("test");
            }
        }
        false
    })
}

#[cfg(test)]
mod tests {
    use super::has_test_usage;

    #[test]
    fn test_usage_finds_test_usage() {
        let file: syn::File =
            syn::parse_str("#[cfg(test)] mod tests { #[test] fn test_usage() { assert!(true); } }")
                .unwrap();
        assert!(has_test_usage(&file));
    }

    #[test]
    fn test_usage_missing() {
        let file: syn::File = syn::parse_str("pub fn compute(x: u32) -> u32 { x * 2 }").unwrap();
        assert!(!has_test_usage(&file));
    }
}
