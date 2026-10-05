use crate::scan;

#[must_use]
pub fn file_reaches_production(file: &syn::File, graph: &scan::CallGraph) -> bool {
    file.items.iter().any(|item| match item {
        syn::Item::Fn(func) => graph.reaches_production(&func.sig.ident.to_string()),
        syn::Item::Impl(impl_block) => impl_block.items.iter().any(|item| {
            matches!(item, syn::ImplItem::Fn(method) if graph.reaches_production(&method.sig.ident.to_string()))
        }),
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::file_reaches_production;
    use crate::scan;
    use lele_lint::Project;

    #[test]
    fn test_usage() {
        let file = syn::parse_str("pub fn setup() {}").unwrap();
        let project = Project::default();
        let graph = scan::CallGraph::collect(&project);
        assert!(file_reaches_production(&file, &graph));
    }
}
