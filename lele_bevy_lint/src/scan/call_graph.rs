use std::collections::HashMap;
use std::collections::HashSet;

use syn::visit::Visit;

use lele_lint::Project;

use crate::scan;

pub struct CallGraph {
    callees: HashMap<String, Vec<String>>,
    systems: HashSet<String>,
}

impl CallGraph {
    #[must_use]
    pub fn collect(project: &Project) -> Self {
        let mut graph = Self {
            callees: HashMap::new(),
            systems: HashSet::new(),
        };
        for source in project.sources() {
            for item in &source.file.items {
                if scan::is_test_module(item) {
                    continue;
                }
                visit_item(&mut graph, item);
            }
        }
        graph
    }

    #[must_use]
    pub fn reaches_production(&self, owner: &str) -> bool {
        if owner.is_empty() {
            return true;
        }
        let targets = HashSet::from([owner.to_string()]);
        reachable(&self.callees, &self.entry_roots(), &targets)
    }

    // needed helper: production entry points (plus every registered system)
    fn entry_roots(&self) -> Vec<String> {
        let mut roots = vec!["main".to_string(), "build".to_string(), "setup".to_string()];
        roots.extend(self.systems.iter().cloned());
        roots
    }
}

// needed helper: walks one item, recording fn and impl-method callees
fn visit_item(graph: &mut CallGraph, item: &syn::Item) {
    match item {
        syn::Item::Fn(func) => {
            record_call(
                &mut graph.callees,
                &mut graph.systems,
                &func.sig.ident.to_string(),
                &func.block,
            );
        }
        syn::Item::Impl(impl_block) => {
            for item in &impl_block.items {
                if let syn::ImplItem::Fn(method) = item {
                    record_call(
                        &mut graph.callees,
                        &mut graph.systems,
                        &method.sig.ident.to_string(),
                        &method.block,
                    );
                }
            }
        }
        syn::Item::Mod(module) => {
            if let Some((_, inner)) = &module.content {
                for item in inner {
                    visit_item(graph, item);
                }
            }
        }
        _ => {}
    }
}

// needed helper: breadth-first reachability from roots to any target
fn reachable(
    callees: &HashMap<String, Vec<String>>,
    roots: &[String],
    targets: &HashSet<String>,
) -> bool {
    let mut stack: Vec<String> = roots.to_vec();
    let mut seen = HashSet::new();
    while let Some(current) = stack.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }
        if targets.contains(&current) {
            return true;
        }
        if let Some(next) = callees.get(&current) {
            stack.extend(next.iter().cloned());
        }
    }
    false
}

// needed helper: records one fn/method's callees and registered systems
fn record_call(
    callees: &mut HashMap<String, Vec<String>>,
    systems: &mut HashSet<String>,
    name: &str,
    block: &syn::Block,
) {
    let mut calls = Vec::new();
    let mut systems_found = Vec::new();
    {
        let mut collector = CallCollector {
            calls: &mut calls,
            systems: &mut systems_found,
        };
        collector.visit_block(block);
    }
    callees.entry(name.to_string()).or_default().extend(calls);
    systems.extend(systems_found);
}

struct CallCollector<'a> {
    calls: &'a mut Vec<String>,
    systems: &'a mut Vec<String>,
}

impl<'ast> Visit<'ast> for CallCollector<'_> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func
            && let Some(last) = path.path.segments.last()
        {
            self.calls.push(last.ident.to_string());
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if let Some(idents) = scan::system_args(node) {
            self.calls.extend(idents.iter().cloned());
            self.systems.extend(idents);
        } else {
            syn::visit::visit_expr_method_call(self, node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CallGraph;
    use lele_lint::Project;
    use std::path::PathBuf;

    #[test]
    fn test_usage() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("a.rs"),
            syn::parse_str("pub fn build() { setup(); } pub fn setup() {}").unwrap(),
        );
        let graph = CallGraph::collect(&project);
        assert!(graph.reaches_production("setup"));
        assert!(!graph.reaches_production("unrelated"));
    }
}
