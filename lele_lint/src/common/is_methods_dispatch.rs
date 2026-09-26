pub(crate) fn is_methods_dispatch(block: &syn::Block) -> bool {
    if block.stmts.len() != 1 {
        return false;
    }
    let Some(syn::Stmt::Expr(expr, None)) = block.stmts.first() else {
        return false;
    };
    let syn::Expr::Call(call) = expr else {
        return false;
    };
    let syn::Expr::Path(path_expr) = call.func.as_ref() else {
        return false;
    };
    if path_expr.qself.is_some() {
        return false;
    }
    let segs: Vec<_> = path_expr.path.segments.iter().collect();
    if segs.len() < 3 {
        return false;
    }
    let Some(first) = segs.first() else {
        return false;
    };
    if matches!(first.ident.to_string().as_str(), "Self" | "self") {
        return false;
    }
    segs.iter().rev().skip(1).any(|seg| seg.ident == "methods")
}

#[cfg(test)]
mod tests {
    use super::is_methods_dispatch;
    use syn::Block;

    #[test]
    fn test_usage() {
        let crate_rooted: Block =
            syn::parse_str("{ crate::methods::foo::check(self, project) }").unwrap();
        assert!(is_methods_dispatch(&crate_rooted));

        let bare: Block = syn::parse_str("{ methods::foo::check(self) }").unwrap();
        assert!(is_methods_dispatch(&bare));

        let old_style: Block =
            syn::parse_str("{ checkers::foo_check::check(self, project) }").unwrap();
        assert!(!is_methods_dispatch(&old_style));

        let short: Block = syn::parse_str("{ foo(x) }").unwrap();
        assert!(!is_methods_dispatch(&short));

        let const_path: Block = syn::parse_str("{ Self::NAME }").unwrap();
        assert!(!is_methods_dispatch(&const_path));

        let with_let: Block = syn::parse_str("{ let a = 1; foo(a) }").unwrap();
        assert!(!is_methods_dispatch(&with_let));
    }
}
