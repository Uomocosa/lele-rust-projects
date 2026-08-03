use std::collections::HashSet;

use crate::checker::Checker;
use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::project::Project;
use crate::severity::Severity;

use super::no_trivial_accessors_register;
// needed helper: parsing utilities

pub struct NoTrivialAccessors;

impl Checker for NoTrivialAccessors {
    fn name(&self) -> &'static str {
        "no_trivial_accessors"
    }

    fn code(&self) -> &'static str {
        "E010"
    }

    fn check(&self, project: &Project) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (rel_path, file) in &project.parsed_files {
            if file
                .items
                .iter()
                .any(|item| matches!(item, syn::Item::Trait(_)))
            {
                continue;
            }

            let pub_fields = collect_pub_fields(file);

            for item in &file.items {
                if let syn::Item::Impl(impl_block) = item {
                    if impl_block.trait_.is_some() {
                        continue;
                    }
                    for impl_item in &impl_block.items {
                        if let syn::ImplItem::Fn(method) = impl_item {
                            if let Some(field) = is_trivial_accessor(method, &pub_fields) {
                                diags.push(Diagnostic {
                                    file: project.src_dir.join(rel_path),
                                    line: 1,
                                    col: 0,
                                    code: "E010".to_string(),
                                    message: format!(
                                        "trivial accessor `{}` reads pub field `{field}`, access the field directly",
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
}

#[rustfmt::skip]
impl NoTrivialAccessors {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_trivial_accessors_register::register(checkers, config)
    }
}

fn collect_pub_fields(file: &syn::File) -> HashSet<String> {
    let mut fields = HashSet::new();
    for item in &file.items {
        if let syn::Item::Struct(s) = item {
            for field in &s.fields {
                if matches!(field.vis, syn::Visibility::Public(_)) {
                    if let Some(ident) = &field.ident {
                        fields.insert(ident.to_string());
                    }
                }
            }
        }
    }
    fields
}

fn is_trivial_accessor(method: &syn::ImplItemFn, pub_fields: &HashSet<String>) -> Option<String> {
    let sig = &method.sig;

    sig.receiver()?;

    if sig.inputs.len() > 1 {
        return None;
    }

    if method.block.stmts.len() != 1 {
        return None;
    }

    let stmt = &method.block.stmts[0];

    let expr = match stmt {
        syn::Stmt::Expr(expr, _) => expr,
        _ => return None,
    };

    extract_self_field(expr, pub_fields)
}

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
        _ => {}
    }
    None
}

fn is_self_ref(expr: &syn::Expr) -> bool {
    if let syn::Expr::Path(path) = expr {
        return path.path.segments.last().is_some_and(|s| s.ident == "self");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::is_trivial_accessor;
    // no test_usage necessary

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
}
