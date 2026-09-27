pub fn name_span(ident: &syn::Ident) -> (usize, usize, usize) {
    let start = ident.span().start();
    let end = ident.span().end();
    (start.line, start.column, end.column)
}

#[cfg(test)]
mod tests {
    use super::name_span;

    #[test]
    fn test_usage() {
        let file: syn::File = syn::parse_str("pub fn foo() {}\n").unwrap();
        let Some(syn::Item::Fn(f)) = file.items.first() else {
            panic!("expected fn");
        };
        let (line, start, end) = name_span(&f.sig.ident);
        assert_eq!((line, start), (1, 7));
        assert_eq!(end, 10);
    }
}
