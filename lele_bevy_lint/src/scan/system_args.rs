use derive_more::{Deref, DerefMut};
use syn::visit::Visit;

#[must_use]
pub fn system_args(node: &syn::ExprMethodCall) -> Option<Vec<String>> {
    if node.method != "add_systems" {
        return None;
    }
    let mut found = Vec::new();
    {
        let mut collector = SystemCollector(&mut found);
        for arg in node.args.iter().skip(1) {
            collector.visit_expr(arg);
        }
    }
    Some(found)
}

// needed helper: collects the last path segment of every path in an expression
#[derive(Deref, DerefMut)]
struct SystemCollector<'a>(&'a mut Vec<String>);

impl<'ast> Visit<'ast> for SystemCollector<'_> {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        if let Some(last) = node.segments.last() {
            self.push(last.ident.to_string());
        }
        syn::visit::visit_path(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::system_args;

    fn method_call(source: &str) -> syn::ExprMethodCall {
        let expr: syn::Expr = syn::parse_str(source).unwrap();
        let syn::Expr::MethodCall(call) = expr else {
            panic!("expected a method call");
        };
        call
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            system_args(&method_call("app.add_systems(Update, (tick, sync))")),
            Some(vec!["tick".to_string(), "sync".to_string()])
        );
        assert_eq!(
            system_args(&method_call("app.add_systems(Startup, setup)")),
            Some(vec!["setup".to_string()])
        );
        assert!(system_args(&method_call("app.add_plugins((a, b))")).is_none());
    }
}
