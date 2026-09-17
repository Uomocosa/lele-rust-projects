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
    let name = fun.sig.ident.to_string();
    fun.block.stmts.insert(
        0,
        syn::parse_quote! {
            let _telegram_guard = telegram_bot::guard::Guard::start(#name);
        },
    );
    quote!(#fun).into()
}

// no test_usage necessary
