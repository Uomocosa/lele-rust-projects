pub(crate) fn is_two_segment_dispatch(block: &syn::Block) -> bool {
    if block.stmts.len() != 1 {
        return false;
    }
    if let syn::Stmt::Expr(syn::Expr::Call(call), _) = &block.stmts[0] {
        if let syn::Expr::Path(path) = call.func.as_ref() {
            return path.path.segments.len() == 2;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::is_two_segment_dispatch;
    use syn::Block;

    #[test]
    fn test_usage() {
        let two_segment: Block = syn::parse_str("{ config_new::new() }").unwrap();
        assert!(is_two_segment_dispatch(&two_segment));

        let three_segment: Block = syn::parse_str("{ crate::clicker::config_new::new() }").unwrap();
        assert!(!is_two_segment_dispatch(&three_segment));

        let real_body: Block = syn::parse_str("{ Self { x: 1 } }").unwrap();
        assert!(!is_two_segment_dispatch(&real_body));

        let multi_stmt: Block = syn::parse_str("{ let x = 1; config_new::new() }").unwrap();
        assert!(!is_two_segment_dispatch(&multi_stmt));
    }
}
