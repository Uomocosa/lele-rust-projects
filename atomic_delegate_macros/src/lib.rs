use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::spanned::Spanned;

const RESERVED_DELEGATE_METHODS: &[&str] = &["new"];

#[proc_macro_attribute]
pub fn atomic_delegate(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "`#[atomic_delegate]` takes no arguments",
        )
        .to_compile_error()
        .into();
    }
    match expand(item.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand(item: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let impl_block: syn::ItemImpl = syn::parse2(item)?;

    if let Some((_, trait_path, _)) = &impl_block.trait_ {
        return Err(syn::Error::new(
            trait_path.span(),
            "`#[atomic_delegate]` only supports inherent impls",
        ));
    }

    let type_ident = last_type_ident(&impl_block.self_ty)?;
    let module = format_ident!("{}", to_snake_case(&type_ident.to_string()));

    let mut out = impl_block.clone();
    out.attrs.push(syn::parse_quote!(#[rustfmt::skip]));
    out.items.clear();

    for item in &impl_block.items {
        let syn::ImplItem::Fn(method) = item else {
            return Err(syn::Error::new(
                item.span(),
                "`#[atomic_delegate]` impl may only contain functions",
            ));
        };
        if !method.block.stmts.is_empty() {
            return Err(syn::Error::new(
                method.block.span(),
                "`#[atomic_delegate]` method body must be an empty placeholder `{}`",
            ));
        }
        let name = &method.sig.ident;
        if RESERVED_DELEGATE_METHODS.contains(&name.to_string().as_str()) {
            return Err(syn::Error::new(
                name.span(),
                "`new` is a constructor and must be defined in the struct file, not delegated",
            ));
        }
        let mut args: Vec<proc_macro2::TokenStream> = Vec::new();
        for input in &method.sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => args.push(quote!(self)),
                syn::FnArg::Typed(typed) => {
                    let syn::Pat::Ident(pat) = &*typed.pat else {
                        return Err(syn::Error::new(
                            typed.pat.span(),
                            "`#[atomic_delegate]` parameters must be plain identifiers",
                        ));
                    };
                    if pat.by_ref.is_some() || pat.subpat.is_some() {
                        return Err(syn::Error::new(
                            pat.span(),
                            "`#[atomic_delegate]` parameters must be plain identifiers",
                        ));
                    }
                    let ident = &pat.ident;
                    args.push(quote!(#ident));
                }
            }
        }
        let call = quote!(crate::methods::#module::#name(#(#args),*));
        let body = if method.sig.asyncness.is_some() {
            quote!(#call.await)
        } else {
            call
        };
        let mut delegate = method.clone();
        delegate.block = syn::parse_quote!({ #body });
        out.items.push(syn::ImplItem::Fn(delegate));
    }

    Ok(quote!(#out))
}

// needed helper: last path segment of an impl self type
fn last_type_ident(ty: &syn::Type) -> syn::Result<syn::Ident> {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return Ok(segment.ident.clone());
    }
    Err(syn::Error::new(
        ty.span(),
        "`#[atomic_delegate]` requires a named type",
    ))
}

// needed helper: PascalCase -> snake_case for the methods module name
fn to_snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::new();
    for (i, c) in chars.iter().enumerate() {
        if !c.is_uppercase() {
            out.push(*c);
            continue;
        }
        let prev = i.checked_sub(1).and_then(|j| chars.get(j));
        let next = chars.get(i.saturating_add(1));
        let needs_sep = prev.is_some_and(|p| p.is_lowercase() || p.is_ascii_digit())
            || (prev.is_some_and(|p| p.is_uppercase()) && next.is_some_and(|n| n.is_lowercase()));
        if needs_sep {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{expand, to_snake_case};
    use quote::quote;

    #[test]
    fn test_usage() {
        let input = quote! {
            impl ClickCounter {
                pub fn increment(&mut self) {}
                pub fn add(&mut self, delta: i32) {}
                pub async fn poll(&mut self) {}
            }
        };
        let expanded = expand(input).unwrap().to_string();
        assert!(expanded.contains("crate :: methods :: click_counter :: increment (self)"));
        assert!(expanded.contains("crate :: methods :: click_counter :: add (self , delta)"));
        assert!(expanded.contains("crate :: methods :: click_counter :: poll (self) . await"));
        assert!(expanded.contains("# [rustfmt :: skip]"));
    }

    #[test]
    fn test_usage_rejects_real_body() {
        let input = quote! { impl Foo { fn f(&self) { let x = 1; } } };
        assert!(expand(input).is_err());
    }

    #[test]
    fn test_usage_rejects_trait_impl() {
        let input = quote! { impl Default for Foo { fn default() -> Self { Foo } } };
        assert!(expand(input).is_err());
    }

    #[test]
    fn test_usage_rejects_reserved_new() {
        let input = quote! { impl Foo { pub fn new() -> Self {} } };
        assert!(expand(input).is_err());
    }

    #[test]
    fn test_usage_snake_case() {
        assert_eq!(to_snake_case("ClickCounter"), "click_counter");
        assert_eq!(to_snake_case("GamePlugin"), "game_plugin");
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
        assert_eq!(to_snake_case("A"), "a");
    }
}
