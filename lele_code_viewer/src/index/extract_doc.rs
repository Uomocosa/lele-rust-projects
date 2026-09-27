use syn::Attribute;
use syn::Expr;
use syn::Lit;
use syn::Meta;

pub fn extract_doc(attrs: &[Attribute]) -> Option<String> {
    let mut lines = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let Meta::NameValue(nv) = &attr.meta
            && let Expr::Lit(el) = &nv.value
            && let Lit::Str(s) = &el.lit
        {
            lines.push(s.value().trim().to_string());
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::extract_doc;

    #[test]
    fn test_usage() {
        let file: syn::File = syn::parse_str("/// hello\n/// world\npub fn f() {}\n").unwrap();
        let Some(syn::Item::Fn(f)) = file.items.first() else {
            panic!("expected fn");
        };
        assert_eq!(extract_doc(&f.attrs).as_deref(), Some("hello\nworld"));
    }
}
