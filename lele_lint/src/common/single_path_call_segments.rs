pub(crate) fn single_path_call_segments(block: &syn::Block) -> Option<Vec<String>> {
    let [syn::Stmt::Expr(syn::Expr::Call(call), None)] = block.stmts.as_slice() else {
        return None;
    };
    let syn::Expr::Path(path_expr) = call.func.as_ref() else {
        return None;
    };
    if path_expr.qself.is_some() {
        return None;
    }
    Some(
        path_expr
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::single_path_call_segments;

    #[test]
    fn test_usage() {
        let block: syn::Block = syn::parse_str("{ foo_bar::run(self, x) }").unwrap();
        assert_eq!(
            single_path_call_segments(&block),
            Some(vec!["foo_bar".to_string(), "run".to_string()])
        );

        let not_a_call: syn::Block = syn::parse_str("{ let a = 1; }").unwrap();
        assert_eq!(single_path_call_segments(&not_a_call), None);
    }
}
