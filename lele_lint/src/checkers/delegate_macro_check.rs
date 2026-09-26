use std::path::Path;

use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

const RESERVED_DELEGATE_METHODS: &[&str] = &["new"];

pub(crate) fn check(
    _self: &checkers::delegate_macro::DelegateMacro,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for (rel_path, file) in &project.parsed_files {
        let Some(stem) = rel_path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(primary) = common::primary_type_name(file, stem) else {
            continue;
        };
        for item in &file.items {
            let syn::Item::Impl(impl_block) = item else {
                continue;
            };
            if impl_block.trait_.is_some() {
                continue;
            }
            if common::self_type_last(&impl_block.self_ty).as_deref() != Some(primary.as_str()) {
                continue;
            }
            if !has_fn(impl_block) {
                continue;
            }

            if common::has_atomic_delegates(&impl_block.attrs) {
                for ident in fn_idents(impl_block) {
                    if is_reserved(ident) {
                        diags.push(diag(
                            project,
                            rel_path,
                            format!(
                                "`{ident}` is a constructor and must be defined in the struct file, not delegated"
                            ),
                        ));
                    }
                }
                continue;
            }

            if all_methods_are_real_reserved(impl_block) {
                continue;
            }
            let type_snake = common::to_snake_case(&primary);
            let mirrors = fn_idents(impl_block)
                .iter()
                .map(|ident| format!("methods/{type_snake}/{ident}.rs"))
                .collect::<Vec<_>>()
                .join(", ");
            diags.push(diag(
                project,
                rel_path,
                format!(
                    "inherent impl for `{primary}` must be annotated with `#[atomic_delegates]`; empty each body and move it to {mirrors} (one `pub fn` + `test_usage` per file)"
                ),
            ));
        }
    }
    diags
}

// needed helper: build an E032 diagnostic anchored to a src file
fn diag(project: &Project, rel_path: &Path, message: String) -> Diagnostic {
    Diagnostic {
        file: project.src_dir.join(rel_path),
        line: 1,
        col: 0,
        code: "E032".to_string(),
        message,
        severity: Severity::Error,
    }
}

// needed helper: impl block contains at least one function
fn has_fn(impl_block: &syn::ItemImpl) -> bool {
    impl_block
        .items
        .iter()
        .any(|item| matches!(item, syn::ImplItem::Fn(_)))
}

// needed helper: method idents of an inherent impl
fn fn_idents(impl_block: &syn::ItemImpl) -> Vec<&syn::Ident> {
    impl_block
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(method) => Some(&method.sig.ident),
            _ => None,
        })
        .collect()
}

// needed helper: reserved delegate method name check
fn is_reserved(ident: &syn::Ident) -> bool {
    RESERVED_DELEGATE_METHODS.contains(&ident.to_string().as_str())
}

// needed helper: every method is a real (non-delegate) reserved constructor
fn all_methods_are_real_reserved(impl_block: &syn::ItemImpl) -> bool {
    let mut has_fn = false;
    for item in &impl_block.items {
        let syn::ImplItem::Fn(method) = item else {
            continue;
        };
        has_fn = true;
        if !is_reserved(&method.sig.ident) {
            return false;
        }
        if method.block.stmts.is_empty() || common::is_delegate_call(&method.block) {
            return false;
        }
    }
    has_fn
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::delegate_macro::DelegateMacro;
    use super::{all_methods_are_real_reserved, check, is_reserved};
    use crate::Project;

    #[test]
    fn test_usage() {
        let new_ident: syn::Ident = syn::parse_str("new").unwrap();
        let other: syn::Ident = syn::parse_str("increment").unwrap();
        assert!(is_reserved(&new_ident));
        assert!(!is_reserved(&other));

        let real: syn::ItemImpl = syn::parse_quote! {
            impl Foo { pub const fn new() -> Self { Self } }
        };
        assert!(all_methods_are_real_reserved(&real));

        let delegated: syn::ItemImpl = syn::parse_quote! {
            impl Foo { pub fn new() -> Self { foo_new::new() } }
        };
        assert!(!all_methods_are_real_reserved(&delegated));

        let mixed: syn::ItemImpl = syn::parse_quote! {
            impl Foo { pub fn new() -> Self { Self } pub fn other(&self) {} }
        };
        assert!(!all_methods_are_real_reserved(&mixed));
    }

    #[test]
    fn test_usage_message_names_mirror_files() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Foo { pub fn run(&self) {} pub fn stop(&self) {} }\n",
            )
            .unwrap(),
        );
        let diags = check(&DelegateMacro, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("methods/foo/run.rs"));
        assert!(diags[0].message.contains("methods/foo/stop.rs"));
    }
}

// no test_usage necessary
