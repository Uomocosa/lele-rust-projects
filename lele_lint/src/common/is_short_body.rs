const MAX_INLINE_STATEMENTS: usize = 3;

pub(crate) fn is_short_body(block: &syn::Block) -> bool {
    block.stmts.len() <= MAX_INLINE_STATEMENTS
}

#[cfg(test)]
mod tests {
    use super::is_short_body;
    use syn::Block;

    #[test]
    fn test_usage() {
        let empty: Block = syn::parse_str("{}").unwrap();
        assert!(is_short_body(&empty));

        let one_call: Block = syn::parse_str("{ config_new::new() }").unwrap();
        assert!(is_short_body(&one_call));

        let one_const: Block = syn::parse_str("{ Self::NAME }").unwrap();
        assert!(is_short_body(&one_const));

        let one_if_else: Block =
            syn::parse_str("{ if self.count > 0 { self.count -= 1 } else { self.count = 0 } }")
                .unwrap();
        assert!(is_short_body(&one_if_else));

        let three_stmts: Block = syn::parse_str("{ let a = 1; let b = 2; a + b }").unwrap();
        assert!(is_short_body(&three_stmts));

        let four_stmts: Block =
            syn::parse_str("{ let a = 1; let b = 2; let c = 3; a + b + c }").unwrap();
        assert!(!is_short_body(&four_stmts));
    }
}
