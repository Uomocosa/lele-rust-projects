use std::collections::BTreeSet;

use derive_more::Deref;
use derive_more::DerefMut;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use syn::visit::Visit;

use crate::index;

pub fn item_externals(
    item: &syn::Item,
    aliases: &index::ExternalAliases,
    crate_externs: &BTreeSet<String>,
) -> Vec<String> {
    if matches!(item, syn::Item::Mod(_)) {
        return Vec::new();
    }
    let mut raw = BTreeSet::new();
    {
        let mut visitor = Heads(&mut raw);
        for attr in attrs_of(item) {
            visitor.visit_attribute(attr);
            if attr.path().is_ident("derive") {
                for head in derive_heads(attr) {
                    visitor.insert(head);
                }
            }
        }
        visitor.visit_item(item);
    }
    let mut out: BTreeSet<String> = aliases.glob_crates.clone();
    for head in &raw {
        if let Some(hit) = resolve_head(head, aliases, crate_externs) {
            out.insert(hit);
        }
    }
    out.into_iter().collect()
}

// needed helper: attribute list of an item for explicit attribute visiting
fn attrs_of(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(node) => &node.attrs,
        syn::Item::Enum(node) => &node.attrs,
        syn::Item::ExternCrate(node) => &node.attrs,
        syn::Item::Fn(node) => &node.attrs,
        syn::Item::ForeignMod(node) => &node.attrs,
        syn::Item::Impl(node) => &node.attrs,
        syn::Item::Macro(node) => &node.attrs,
        syn::Item::Mod(node) => &node.attrs,
        syn::Item::Static(node) => &node.attrs,
        syn::Item::Struct(node) => &node.attrs,
        syn::Item::Trait(node) => &node.attrs,
        syn::Item::TraitAlias(node) => &node.attrs,
        syn::Item::Type(node) => &node.attrs,
        syn::Item::Union(node) => &node.attrs,
        syn::Item::Use(node) => &node.attrs,
        _ => &[],
    }
}

// needed helper: first segments of paths in a derive attribute list
fn derive_heads(attr: &syn::Attribute) -> Vec<String> {
    let Ok(paths) = attr.parse_args_with(
        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
    ) else {
        return Vec::new();
    };
    paths
        .iter()
        .filter_map(|path| {
            path.segments
                .first()
                .map(|segment| segment.ident.to_string())
        })
        .collect()
}

// needed helper: resolve one path head to its external crate name
fn resolve_head(
    head: &str,
    aliases: &index::ExternalAliases,
    crate_externs: &BTreeSet<String>,
) -> Option<String> {
    if let Some(hit) = aliases.local_to_crate.get(head) {
        return Some(hit.clone());
    }
    if matches!(
        head,
        "proc_macro" | "proc_macro_attribute" | "proc_macro_derive"
    ) {
        return Some("proc_macro".to_string());
    }
    if matches!(head, "self" | "Self" | "super" | "crate") {
        return None;
    }
    if crate_externs.contains(head) {
        return Some(head.to_string());
    }
    None
}

#[derive(Deref, DerefMut)]
struct Heads<'a>(&'a mut BTreeSet<String>);

impl<'ast> syn::visit::Visit<'ast> for Heads<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(head) = path.segments.first() {
            self.insert(head.ident.to_string());
        }
        syn::visit::visit_path(self, path);
    }
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if let Some(head) = mac.path.segments.first() {
            self.insert(head.ident.to_string());
        }
        collect_token_paths(&mac.tokens, self);
        syn::visit::visit_macro(self, mac);
    }
}

// needed helper: first segments of qualified paths inside macro token streams
fn collect_token_paths(tokens: &TokenStream, heads: &mut BTreeSet<String>) {
    let mut stream = tokens.clone().into_iter().peekable();
    while let Some(token) = stream.next() {
        match token {
            TokenTree::Group(group) => collect_token_paths(&group.stream(), heads),
            TokenTree::Ident(first) => {
                if !skip_colons(&mut stream) {
                    continue;
                }
                let Some(TokenTree::Ident(_)) = stream.peek() else {
                    continue;
                };
                stream.next();
                heads.insert(first.to_string());
            }
            _ => {}
        }
    }
}

// needed helper: consume one double-colon separator from a token stream
fn skip_colons(stream: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>) -> bool {
    let first = matches!(stream.peek(), Some(TokenTree::Punct(mark)) if mark.as_char() == ':');
    if !first {
        return false;
    }
    stream.next();
    let second = matches!(stream.peek(), Some(TokenTree::Punct(mark)) if mark.as_char() == ':');
    if !second {
        return false;
    }
    stream.next();
    true
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::item_externals;
    use crate::index;

    #[test]
    fn test_usage() {
        let file: syn::File = syn::parse_str(
            "use quote::quote;\nuse syn::spanned::Spanned;\nfn render(span: Spanned) -> String { quote!(#span).to_string() }\nfn local() -> usize { 1 }\nfn full() -> proc_macro2::TokenStream { proc_macro2::TokenStream::new() }\nfn quoted() -> proc_macro2::TokenStream { quote::quote!(syn::Error::new_spanned()) }\n",
        )
        .unwrap();
        let aliases = index::collect_external_aliases(&file);
        let deps: BTreeSet<String> = BTreeSet::from([
            "proc_macro2".to_string(),
            "syn".to_string(),
            "quote".to_string(),
        ]);
        let mut bodies: Vec<Vec<String>> = Vec::new();
        for item in &file.items {
            bodies.push(item_externals(item, &aliases, &deps));
        }
        assert!(bodies[2].contains(&"syn".to_string()));
        assert!(bodies[2].contains(&"quote".to_string()));
        assert_eq!(bodies[3], Vec::<String>::new());
        assert!(bodies[4].contains(&"proc_macro2".to_string()));
        assert!(bodies[5].contains(&"quote".to_string()));
        assert!(bodies[5].contains(&"syn".to_string()));
        assert!(bodies[5].contains(&"proc_macro2".to_string()));
    }
}
