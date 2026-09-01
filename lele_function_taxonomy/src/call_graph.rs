use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct CallGraph {
    pub edges: HashMap<String, Vec<String>>,
    pub reverse: HashMap<String, Vec<String>>,
}

impl CallGraph {
    pub fn add_edge(&mut self, caller: String, callee: String) {
        self.edges
            .entry(caller.clone())
            .or_default()
            .push(callee.clone());
        self.reverse.entry(callee).or_default().push(caller);
    }

    pub fn callees(&self, caller: &str) -> &[String] {
        self.edges.get(caller).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn callers(&self, callee: &str) -> &[String] {
        self.reverse.get(callee).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn all_nodes(&self) -> Vec<String> {
        let mut nodes: std::collections::HashSet<String> = self.edges.keys().cloned().collect();
        for v in self.edges.values() {
            for callee in v {
                nodes.insert(callee.clone());
            }
        }
        nodes.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::CallGraph;

    #[test]
    fn test_usage() {
        let mut g = CallGraph::default();
        g.add_edge("main".to_string(), "render_frame".to_string());
        g.add_edge("render_frame".to_string(), "draw".to_string());
        assert_eq!(g.callees("main"), &["render_frame".to_string()]);
        assert_eq!(g.callers("draw"), &["render_frame".to_string()]);
        assert_eq!(g.all_nodes().len(), 3);
    }
}
