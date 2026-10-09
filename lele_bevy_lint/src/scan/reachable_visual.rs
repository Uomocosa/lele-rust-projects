use crate::scan;

#[must_use]
pub fn reachable_visual(file: &syn::File, graph: &scan::CallGraph) -> Option<scan::FoundVisual> {
    if !scan::file_reaches_production(file, graph) {
        return None;
    }
    scan::prod_visuals(file)
        .into_iter()
        .find(|found| graph.reaches_production(&found.owner))
}

#[cfg(test)]
mod tests {
    use super::reachable_visual;
    use crate::scan;
    use lele_lint::Project;

    #[test]
    fn test_usage() {
        let graph = scan::CallGraph::collect(&Project::default());
        let file = syn::parse_str(
            "pub struct Spawner;
             pub fn setup(spawner: &mut Spawner) { spawner.spawn((Node, Text::new(\"x\"))); }",
        )
        .unwrap();
        let visual = reachable_visual(&file, &graph).unwrap();
        assert_eq!(visual.visual, "Node");
        assert_eq!(visual.owner, "setup");

        let no_ui = syn::parse_str("pub fn setup(c: &mut S) { c.spawn((Camera2d,)); }").unwrap();
        assert!(reachable_visual(&no_ui, &graph).is_none());
    }
}
