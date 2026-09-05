pub(crate) fn is_delegate_call(block: &syn::Block) -> bool {
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
    if segs.len() < 2 {
        return false;
    }
    let Some(first_seg) = segs.first() else {
        return false;
    };
    let first = first_seg.ident.to_string();
    !matches!(first.as_str(), "Self" | "self" | "crate")
}

#[cfg(test)]
mod tests {
    use super::is_delegate_call;
    use syn::Block;

    #[test]
    fn test_usage() {
        let delegate: Block = syn::parse_str("{ config_new::new() }").unwrap();
        assert!(is_delegate_call(&delegate));

        let with_args: Block = syn::parse_str("{ foo_bar::run(self, x) }").unwrap();
        assert!(is_delegate_call(&with_args));

        let struct_lit: Block = syn::parse_str("{ Self { x: 1 } }").unwrap();
        assert!(!is_delegate_call(&struct_lit));

        let self_call: Block = syn::parse_str("{ Self::new() }").unwrap();
        assert!(!is_delegate_call(&self_call));

        let with_let: Block = syn::parse_str("{ let a = 1; Foo { x: a } }").unwrap();
        assert!(!is_delegate_call(&with_let));
    }
}
