use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectCause {
    IoRoot(String),
    HiddenStaticRead(String),
    HiddenStaticWrite(String),
    ThreadLocal(String),
    DeclaredDishonest(String),
}

impl std::fmt::Display for DirectCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoRoot(path) => write!(f, "calls `{path}` (hidden read/write)"),
            Self::HiddenStaticRead(name) => write!(f, "reads non-Freeze static `{name}`"),
            Self::HiddenStaticWrite(name) => write!(f, "writes `static mut {name}`"),
            Self::ThreadLocal(name) => write!(f, "accesses thread-local `{name}`"),
            Self::DeclaredDishonest(name) => write!(f, "`{name}` is declared dishonest"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub path: String,
    pub direct: Option<DirectCause>,
}

#[derive(Debug, Default, Clone)]
pub struct HonestyGraph {
    pub nodes: BTreeMap<NodeId, Node>,
    edges: BTreeSet<(NodeId, NodeId)>,
    reverse: BTreeMap<NodeId, BTreeSet<NodeId>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    pub chain: Vec<NodeId>,
    pub cause: DirectCause,
}

impl HonestyGraph {
    pub fn add_node(&mut self, id: NodeId, path: String, direct: Option<DirectCause>) {
        self.nodes.entry(id).or_insert(Node { id, path, direct });
    }

    pub fn add_edge(&mut self, caller: NodeId, callee: NodeId) {
        if self.edges.insert((caller, callee)) {
            self.reverse.entry(callee).or_default().insert(caller);
        }
    }

    pub fn callees(&self, caller: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.edges
            .iter()
            .filter(move |(c, _)| *c == caller)
            .map(|(_, callee)| *callee)
    }

    pub fn path_of(&self, id: NodeId) -> &str {
        self.nodes.get(&id).map_or("?", |n| n.path.as_str())
    }

    // needed helper: deterministic worklist fixpoint; returns shortest witness per dishonest node
    pub fn classify(&self) -> BTreeMap<NodeId, Witness> {
        let mut witnesses: BTreeMap<NodeId, Witness> = BTreeMap::new();
        let mut queue: VecDeque<NodeId> = VecDeque::new();

        let mut ordered: Vec<NodeId> = self.nodes.keys().copied().collect();
        ordered.sort();
        for id in &ordered {
            if let Some(cause) = self.nodes.get(id).and_then(|n| n.direct.clone()) {
                witnesses.insert(
                    *id,
                    Witness {
                        chain: vec![*id],
                        cause,
                    },
                );
                queue.push_back(*id);
            }
        }

        while let Some(node) = queue.pop_front() {
            let mut callers: Vec<NodeId> = self
                .reverse
                .get(&node)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default();
            callers.sort();
            for caller in callers {
                if witnesses.contains_key(&caller) {
                    continue;
                }
                let Some(base) = witnesses.get(&node) else {
                    continue;
                };
                let mut chain = vec![caller];
                chain.extend(base.chain.iter().copied());
                witnesses.insert(
                    caller,
                    Witness {
                        chain,
                        cause: base.cause.clone(),
                    },
                );
                queue.push_back(caller);
            }
        }

        witnesses
    }
}

#[cfg(test)]
mod tests {
    use super::{DirectCause, HonestyGraph, NodeId};

    #[test]
    fn test_usage() {
        assert_eq!(NodeId(1), NodeId(1));
    }

    #[test]
    fn test_usage_cycle_terminates() {
        let mut g = HonestyGraph::default();
        g.add_node(NodeId(0), "a".to_string(), None);
        g.add_node(NodeId(1), "b".to_string(), None);
        g.add_edge(NodeId(0), NodeId(1));
        g.add_edge(NodeId(1), NodeId(0));
        g.add_node(
            NodeId(2),
            "clock".to_string(),
            Some(DirectCause::IoRoot(
                "std::time::SystemTime::now".to_string(),
            )),
        );
        g.add_edge(NodeId(1), NodeId(2));
        let w = g.classify();
        assert_eq!(w.len(), 3);
        assert_eq!(w.get(&NodeId(0)).map(|x| x.chain.len()), Some(3));
    }

    #[test]
    fn test_usage_witness_shortest() {
        let mut g = HonestyGraph::default();
        g.add_node(NodeId(0), "top".to_string(), None);
        g.add_node(
            NodeId(1),
            "leaf".to_string(),
            Some(DirectCause::HiddenStaticWrite("M".to_string())),
        );
        g.add_edge(NodeId(0), NodeId(1));
        let w = g.classify();
        assert_eq!(
            w.get(&NodeId(0)).map(|x| x.chain.clone()),
            Some(vec![NodeId(0), NodeId(1)])
        );
    }
}
