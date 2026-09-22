use super::delegate_macro::DelegateMacro;
use crate::common;
use crate::Diagnostic;
use crate::Layout;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &DelegateMacro, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if project.layout != Layout::Methods {
        return diags;
    }
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
            if common::has_atomic_delegate(&impl_block.attrs) {
                continue;
            }
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line: 1,
                col: 0,
                code: "E032".to_string(),
                message: format!(
                    "inherent impl for `{primary}` must be annotated with `#[atomic_delegate]`"
                ),
                severity: Severity::Error,
            });
        }
    }
    diags
}

// needed helper: impl block contains at least one function
fn has_fn(impl_block: &syn::ItemImpl) -> bool {
    impl_block
        .items
        .iter()
        .any(|item| matches!(item, syn::ImplItem::Fn(_)))
}

// no test_usage necessary
