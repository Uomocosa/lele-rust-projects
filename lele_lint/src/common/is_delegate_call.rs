use crate::common;

pub(crate) fn is_delegate_call(block: &syn::Block) -> bool {
    let Some(segments) = common::single_path_call_segments(block) else {
        return false;
    };
    if segments.len() < 2 {
        return false;
    }
    let Some(first) = segments.first() else {
        return false;
    };
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
