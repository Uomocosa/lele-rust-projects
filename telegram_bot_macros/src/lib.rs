use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

#[must_use]
#[proc_macro_attribute]
pub fn telegram_notify(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut fun = match syn::parse::<ItemFn>(item) {
        Ok(fun) => fun,
        Err(err) => return err.to_compile_error().into(),
    };
    prepend_guard(&mut fun);
    quote!(#fun).into()
}

// needed helper: inserts the telegram guard as the first statement of the fn
fn prepend_guard(fun: &mut ItemFn) {
    let name = fun.sig.ident.to_string();
    fun.block.stmts.insert(
        0,
        syn::parse_quote! {
            let _telegram_guard = telegram_bot::guard::Guard::start(#name);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::prepend_guard;
    use syn::ItemFn;

    #[test]
    fn test_usage() {
        let mut fun: ItemFn = syn::parse_quote! {
            fn sample() {}
        };
        prepend_guard(&mut fun);
        assert_eq!(fun.block.stmts.len(), 1);
        let rendered = quote::quote!(#fun).to_string();
        assert!(rendered.contains("Guard :: start"));
    }
}
