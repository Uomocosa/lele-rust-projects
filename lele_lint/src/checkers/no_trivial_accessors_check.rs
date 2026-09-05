use std::collections::HashMap;
use std::collections::HashSet;

use super::no_trivial_accessors::NoTrivialAccessors;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &NoTrivialAccessors, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        if file
            .items
            .iter()
            .any(|item| matches!(item, syn::Item::Trait(_)))
        {
            continue;
        }

        let vis_fields = collect_vis_fields(file);
        let pub_fields: HashSet<String> = vis_fields.keys().cloned().collect();

        for item in &file.items {
            if let syn::Item::Impl(impl_block) = item {
                if impl_block.trait_.is_some() {
                    continue;
                }
                for impl_item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        if is_builder(method) {
                            continue;
                        }
                        if let Some(field) = is_trivial_accessor(method, &pub_fields) {
                            let vis = vis_fields.get(&field).cloned().unwrap_or_default();
                            diags.push(Diagnostic {
                                file: project.src_dir.join(rel_path),
                                line: 1,
                                col: 0,
                                code: "E010".to_string(),
                                message: format!(
                                    "trivial accessor `{}` reads {vis} field `{field}`, make field public/pub(crate) and access directly",
                                    method.sig.ident
                                ),
                                severity: Severity::Error,
                            });
                        } else if let Some(field) = is_trivial_setter(method, &pub_fields) {
                            let vis = vis_fields.get(&field).cloned().unwrap_or_default();
                            diags.push(Diagnostic {
                                file: project.src_dir.join(rel_path),
                                line: 1,
                                col: 0,
                                code: "E010".to_string(),
                                message: format!(
                                    "trivial setter `{}` assigns {vis} field `{field}`, make field public/pub(crate) and assign directly",
                                    method.sig.ident
                                ),
                                severity: Severity::Error,
                            });
                        }
                    }
                }
            }
        }
    }

    diags
}

// needed helper: visible field collection — pub + all Visibility::Restricted
fn vis_to_string(vis: &syn::Visibility) -> Option<String> {
    match vis {
        syn::Visibility::Public(_) => Some("pub".to_string()),
        syn::Visibility::Restricted(res) => {
            let s = quote::quote!(#res).to_string();
            Some(s.replace(" )", ")").replace("( ", "("))
        }
        syn::Visibility::Inherited => None,
    }
}

// needed helper: visible field map field -> vis string
fn collect_vis_fields(file: &syn::File) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    for item in &file.items {
        if let syn::Item::Struct(s) = item {
            for field in &s.fields {
                if let Some(vis_str) = vis_to_string(&field.vis) {
                    if let Some(ident) = &field.ident {
                        fields.insert(ident.to_string(), vis_str);
                    }
                }
            }
        }
    }
    fields
}

// needed helper: builder with_* detection — don't flag
fn is_builder(method: &syn::ImplItemFn) -> bool {
    let name = method.sig.ident.to_string();
    if !name.starts_with("with_") {
        return false;
    }
    let Some(receiver) = method.sig.receiver() else {
        return false;
    };
    // receiver is `self` by value (no reference, no mut)
    if receiver.reference.is_some() {
        return false;
    }
    // must have exactly one extra param and return Self
    if method.sig.inputs.len() != 2 {
        return false;
    }
    // return type contains Self
    match &method.sig.output {
        syn::ReturnType::Type(_, ty) => {
            let s = quote::quote!(#ty).to_string();
            s.contains("Self")
        }
        syn::ReturnType::Default => false,
    }
}

// needed helper: log macro stripping
fn is_log_macro(stmt: &syn::Stmt) -> bool {
    let mac = match stmt {
        syn::Stmt::Macro(m) => &m.mac,
        syn::Stmt::Expr(syn::Expr::Macro(em), _) => &em.mac,
        _ => return false,
    };
    let segments: Vec<String> = mac
        .path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect();
    let path = segments.join("::");
    if path == "println" || path == "eprintln" {
        return true;
    }
    if path.ends_with("::println") || path.ends_with("::eprintln") {
        return true;
    }
    let last = segments.last().cloned().unwrap_or_default();
    let is_log_ident = matches!(last.as_str(), "info" | "debug" | "warn" | "error" | "trace");
    if !is_log_ident {
        return false;
    }
    segments
        .iter()
        .any(|s| s == "tracing" || s == "log" || s == "tracing_subscriber")
}

fn filtered_stmts(block: &syn::Block) -> Vec<&syn::Stmt> {
    block.stmts.iter().filter(|s| !is_log_macro(s)).collect()
}

// needed helper: trivial accessor pattern detection
fn is_trivial_accessor(method: &syn::ImplItemFn, pub_fields: &HashSet<String>) -> Option<String> {
    let sig = &method.sig;

    let receiver = sig.receiver()?;
    // &self (not &mut, not self by value)
    if receiver.reference.is_none() || receiver.mutability.is_some() {
        return None;
    }

    if sig.inputs.len() != 1 {
        return None;
    }

    let stmts = filtered_stmts(&method.block);
    let [stmt] = stmts.as_slice() else {
        return None;
    };

    let expr = match stmt {
        syn::Stmt::Expr(expr, _) => expr,
        _ => return None,
    };

    extract_self_field(expr, pub_fields)
}

// needed helper: trivial setter detection
fn is_trivial_setter(method: &syn::ImplItemFn, pub_fields: &HashSet<String>) -> Option<String> {
    let sig = &method.sig;

    let receiver = sig.receiver()?;
    if receiver.reference.is_none() || receiver.mutability.is_none() {
        return None;
    }

    if sig.inputs.len() != 2 {
        return None;
    }

    let stmts = filtered_stmts(&method.block);
    let [stmt] = stmts.as_slice() else {
        return None;
    };
    let expr = match stmt {
        syn::Stmt::Expr(expr, _) => expr,
        _ => return None,
    };

    if let syn::Expr::Assign(assign) = expr {
        return extract_assign_field(assign, sig, pub_fields);
    }

    None
}

// needed helper: extract field from self.field = param assign
fn extract_assign_field(
    assign: &syn::ExprAssign,
    sig: &syn::Signature,
    pub_fields: &HashSet<String>,
) -> Option<String> {
    // left must be self.field
    let field_name = match &*assign.left {
        syn::Expr::Field(field) => {
            if let syn::Member::Named(named) = &field.member {
                let n = named.to_string();
                if is_self_ref(&field.base) && pub_fields.contains(&n) {
                    n
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        _ => {
            return None;
        }
    };

    let param_ident = sig.inputs.iter().nth(1).and_then(|arg| match arg {
        syn::FnArg::Typed(pat) => {
            if let syn::Pat::Ident(id) = &*pat.pat {
                Some(id.ident.to_string())
            } else {
                None
            }
        }
        syn::FnArg::Receiver(_) => None,
    })?;

    let right = &*assign.right;
    let right_str = quote::quote!(#right).to_string().replace(' ', "");
    let param_str = param_ident.replace(' ', "");
    // allow `v`, `v.clone()`, `v.to_owned()` as trivial RHS
    if right_str == param_str
        || right_str == format!("{param_str}.clone()")
        || right_str == format!("{param_str}.to_owned()")
    {
        return Some(field_name);
    }
    // clone_from pattern: not assign, handled separately — keep simple
    None
}

// needed helper: self.field expression extraction
fn extract_self_field(expr: &syn::Expr, pub_fields: &HashSet<String>) -> Option<String> {
    match expr {
        syn::Expr::Field(field) => {
            if let syn::Member::Named(named) = &field.member {
                let field_name = named.to_string();
                if is_self_ref(&field.base) && pub_fields.contains(&field_name) {
                    return Some(field_name);
                }
            }
        }
        syn::Expr::Reference(ref_expr) => {
            return extract_self_field(&ref_expr.expr, pub_fields);
        }
        syn::Expr::MethodCall(call) if call.method == "clone" && call.args.is_empty() => {
            return extract_self_field(&call.receiver, pub_fields);
        }
        _ => {}
    }
    None
}

// needed helper: self-reference expression check
fn is_self_ref(expr: &syn::Expr) -> bool {
    if let syn::Expr::Path(path) = expr {
        return path.path.segments.last().is_some_and(|s| s.ident == "self");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{is_trivial_accessor, is_trivial_setter};
    use std::collections::HashSet;

    #[test]
    fn test_usage_flags_trivial_getter() {
        let method: syn::ImplItemFn =
            syn::parse_str("fn value(&self) -> u32 { self.value }").unwrap();
        let mut fields = HashSet::new();
        fields.insert("value".into());
        assert!(is_trivial_accessor(&method, &fields).is_some());
    }

    #[test]
    fn test_usage_allows_computation() {
        let method: syn::ImplItemFn =
            syn::parse_str("fn doubled(&self) -> u32 { self.value * 2 }").unwrap();
        let mut fields = HashSet::new();
        fields.insert("value".into());
        assert!(is_trivial_accessor(&method, &fields).is_none());
    }

    #[test]
    fn test_usage_allows_non_pub() {
        let method: syn::ImplItemFn =
            syn::parse_str("fn inner(&self) -> u32 { self.secret }").unwrap();
        let fields: HashSet<String> = HashSet::new();
        assert!(is_trivial_accessor(&method, &fields).is_none());
    }

    #[test]
    fn test_usage_flags_getter_with_log() {
        let method: syn::ImplItemFn =
            syn::parse_str("fn value(&self) -> u32 { tracing::info!(\"get\"); self.value }")
                .unwrap();
        let mut fields = HashSet::new();
        fields.insert("value".into());
        assert!(is_trivial_accessor(&method, &fields).is_some());
    }

    #[test]
    fn test_usage_flags_setter() {
        let method: syn::ImplItemFn =
            syn::parse_str("fn set_value(&mut self, v: u32) { self.value = v; }").unwrap();
        let mut fields = HashSet::new();
        fields.insert("value".into());
        assert!(is_trivial_setter(&method, &fields).is_some());
    }

    #[test]
    fn test_usage_flags_setter_with_log() {
        let method: syn::ImplItemFn = syn::parse_str(
            "fn set_value(&mut self, v: u32) { self.value = v; println!(\"set\"); }",
        )
        .unwrap();
        let mut fields = HashSet::new();
        fields.insert("value".into());
        assert!(is_trivial_setter(&method, &fields).is_some());
    }

    #[test]
    fn test_usage_allows_builder() {
        let method: syn::ImplItemFn =
            syn::parse_str("fn with_tag(self, tag: u64) -> Self { Self { tag, ..self } }").unwrap();
        assert!(super::is_builder(&method));
    }
}

// no test_usage necessary
