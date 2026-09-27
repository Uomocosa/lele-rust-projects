use std::path::Path;

use proc_macro2::Span;
use syn::spanned::Spanned;

use crate::index;

pub fn collect_file_items(rel: &Path, text: &str, file: &syn::File) -> Vec<index::IndexItem> {
    let module = index::module_of(rel);
    let mut out = Vec::new();
    for item in &file.items {
        collect_item(item, rel, text, &module, None, &mut out);
    }
    out
}

// needed helper: collect one item (recursing into inline modules and impl blocks)
fn collect_item(
    item: &syn::Item,
    rel: &Path,
    text: &str,
    module: &str,
    container: Option<&str>,
    out: &mut Vec<index::IndexItem>,
) {
    match item {
        syn::Item::Fn(f) => push_item(
            rel,
            text,
            module,
            container,
            &f.sig.ident,
            index::ItemKind::Fn,
            &f.attrs,
            f.span(),
            out,
        ),
        syn::Item::Struct(s) => push_item(
            rel,
            text,
            module,
            container,
            &s.ident,
            index::ItemKind::Struct,
            &s.attrs,
            s.span(),
            out,
        ),
        syn::Item::Enum(e) => push_item(
            rel,
            text,
            module,
            container,
            &e.ident,
            index::ItemKind::Enum,
            &e.attrs,
            e.span(),
            out,
        ),
        syn::Item::Const(c) => push_item(
            rel,
            text,
            module,
            container,
            &c.ident,
            index::ItemKind::Const,
            &c.attrs,
            c.span(),
            out,
        ),
        syn::Item::Static(s) => push_item(
            rel,
            text,
            module,
            container,
            &s.ident,
            index::ItemKind::Static,
            &s.attrs,
            s.span(),
            out,
        ),
        syn::Item::Type(t) => push_item(
            rel,
            text,
            module,
            container,
            &t.ident,
            index::ItemKind::TypeAlias,
            &t.attrs,
            t.span(),
            out,
        ),
        syn::Item::Trait(t) => push_item(
            rel,
            text,
            module,
            container,
            &t.ident,
            index::ItemKind::Trait,
            &t.attrs,
            t.span(),
            out,
        ),
        syn::Item::Mod(m) => {
            if let Some((_, items)) = &m.content {
                let nested = if module.is_empty() {
                    m.ident.to_string()
                } else {
                    format!("{module}::{}", m.ident)
                };
                for inner in items {
                    collect_item(inner, rel, text, &nested, None, out);
                }
            }
        }
        syn::Item::Impl(im) => {
            let ty = type_name(&im.self_ty);
            for impl_item in &im.items {
                if let syn::ImplItem::Fn(m) = impl_item {
                    push_item(
                        rel,
                        text,
                        module,
                        ty.as_deref(),
                        &m.sig.ident,
                        index::ItemKind::Fn,
                        &m.attrs,
                        m.span(),
                        out,
                    );
                }
            }
        }
        _ => {}
    }
}

// needed helper: build one IndexItem from its ident/kind/span
fn push_item(
    rel: &Path,
    text: &str,
    module: &str,
    container: Option<&str>,
    ident: &syn::Ident,
    kind: index::ItemKind,
    attrs: &[syn::Attribute],
    span: Span,
    out: &mut Vec<index::IndexItem>,
) {
    let name = ident.to_string();
    let (name_line, name_col_start, name_col_end) = index::name_span(ident);
    let id = match container {
        Some(c) if module.is_empty() => format!("{c}::{name}"),
        Some(c) => format!("{module}::{c}::{name}"),
        None if module.is_empty() => name.clone(),
        None => format!("{module}::{name}"),
    };
    out.push(index::IndexItem {
        id,
        name,
        module: module.to_string(),
        kind,
        file: rel.to_path_buf(),
        start_line: span.start().line,
        end_line: span.end().line,
        name_line,
        name_col_start,
        name_col_end,
        signature: index::derive_signature(text, name_line),
        doc: index::extract_doc(attrs),
        delegates_to: None,
    });
}

// needed helper: last path segment of an impl self type
fn type_name(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(tp) = ty {
        tp.path.segments.last().map(|s| s.ident.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::collect_file_items;

    #[test]
    fn test_usage() {
        let text = "pub struct Bar;\npub fn top() {}\nimpl Bar { pub fn new() -> Self { Self } }\n";
        let file: syn::File = syn::parse_str(text).unwrap();
        let items = collect_file_items(Path::new("src/foo.rs"), text, &file);
        let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"foo::Bar"));
        assert!(ids.contains(&"foo::top"));
        assert!(ids.contains(&"foo::Bar::new"));
    }
}
