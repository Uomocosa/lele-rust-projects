use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::index;

const MAX_GROUPS_PER_NODE: usize = 3;
const MIN_GROUP_SIZE: usize = 3;
const RESOLUTION: f64 = 1.0;
const MAX_PASSES: usize = 32;
const EPSILON: f64 = 1e-12;

pub fn detect_groups(node_count: usize, edges: &[(usize, usize)]) -> Vec<index::ItemGroup> {
    let nbrs = neighbors(node_count, edges);
    let (persona_of, owner) = split_personas(&nbrs);
    let persona_adj = persona_graph(&nbrs, &persona_of, owner.len());
    let community = louvain(&persona_adj);
    collect_groups(node_count, &owner, &community)
}

// needed helper: sorted, deduplicated undirected neighbour lists
fn neighbors(node_count: usize, edges: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut nbrs: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    for &(a, b) in edges {
        if a == b || a >= node_count || b >= node_count {
            continue;
        }
        if let Some(list) = nbrs.get_mut(a) {
            list.push(b);
        }
        if let Some(list) = nbrs.get_mut(b) {
            list.push(a);
        }
    }
    for list in &mut nbrs {
        list.sort_unstable();
        list.dedup();
    }
    nbrs
}

// needed helper: global persona id per (node, neighbour slot) and persona -> node owner
fn split_personas(nbrs: &[Vec<usize>]) -> (Vec<Vec<usize>>, Vec<usize>) {
    let mut persona_of = Vec::with_capacity(nbrs.len());
    let mut owner: Vec<usize> = Vec::new();
    for (v, list) in nbrs.iter().enumerate() {
        let labels = ego_components(nbrs, list);
        let base = owner.len();
        let count = labels
            .iter()
            .copied()
            .max()
            .map_or(1, |m| m.saturating_add(1));
        for _ in 0..count {
            owner.push(v);
        }
        persona_of.push(labels.iter().map(|l| base.saturating_add(*l)).collect());
    }
    (persona_of, owner)
}

// needed helper: split a neighbourhood into clusters; singletons join the largest cluster
fn ego_components(nbrs: &[Vec<usize>], list: &[usize]) -> Vec<usize> {
    let mut parent: Vec<usize> = (0..list.len()).collect();
    for (i, a) in list.iter().enumerate() {
        let Some(a_nbrs) = nbrs.get(*a) else {
            continue;
        };
        for (j, b) in list.iter().enumerate().skip(i.saturating_add(1)) {
            if a_nbrs.binary_search(b).is_ok() {
                let ra = find(&mut parent, i);
                let rb = find(&mut parent, j);
                if let Some(slot) = parent.get_mut(ra.max(rb)) {
                    *slot = ra.min(rb);
                }
            }
        }
    }
    let roots: Vec<usize> = (0..list.len()).map(|i| find(&mut parent, i)).collect();
    let mut sizes: BTreeMap<usize, usize> = BTreeMap::new();
    for &r in &roots {
        let entry = sizes.entry(r).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    let mut label_of: BTreeMap<usize, usize> = BTreeMap::new();
    for (&root, &size) in &sizes {
        if size >= 2 {
            let next = label_of.len();
            label_of.insert(root, next);
        }
    }
    let largest = sizes
        .iter()
        .filter(|&(_, &s)| s >= 2)
        .max_by_key(|&(&r, &s)| (s, std::cmp::Reverse(r)))
        .and_then(|(r, _)| label_of.get(r).copied())
        .unwrap_or(0);
    roots
        .iter()
        .map(|r| label_of.get(r).copied().unwrap_or(largest))
        .collect()
}

// needed helper: union-find root lookup with path halving
fn find(parent: &mut [usize], start: usize) -> usize {
    let mut x = start;
    while let Some(&p) = parent.get(x) {
        if p == x {
            return x;
        }
        let gp = parent.get(p).copied().unwrap_or(p);
        if let Some(slot) = parent.get_mut(x) {
            *slot = gp;
        }
        x = gp;
    }
    x
}

// needed helper: weighted persona adjacency, hub edges damped by 1/sqrt(deg_u * deg_v)
fn persona_graph(
    nbrs: &[Vec<usize>],
    persona_of: &[Vec<usize>],
    persona_count: usize,
) -> Vec<Vec<(usize, f64)>> {
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); persona_count];
    for (v, list) in nbrs.iter().enumerate() {
        for (slot, &u) in list.iter().enumerate() {
            if u <= v {
                continue;
            }
            let Some(u_list) = nbrs.get(u) else {
                continue;
            };
            let Ok(back) = u_list.binary_search(&v) else {
                continue;
            };
            let pv = persona_of.get(v).and_then(|p| p.get(slot)).copied();
            let pu = persona_of.get(u).and_then(|p| p.get(back)).copied();
            let (Some(pv), Some(pu)) = (pv, pu) else {
                continue;
            };
            let weight = 1.0 / (to_f64(list.len()) * to_f64(u_list.len())).sqrt();
            if let Some(out) = adj.get_mut(pv) {
                out.push((pu, weight));
            }
            if let Some(out) = adj.get_mut(pu) {
                out.push((pv, weight));
            }
        }
    }
    adj
}

// needed helper: lossless-enough usize -> f64 without `as`
fn to_f64(n: usize) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

// needed helper: multilevel Louvain; returns a community per input vertex
fn louvain(adj: &[Vec<(usize, f64)>]) -> Vec<usize> {
    let mut assignment: Vec<usize> = (0..adj.len()).collect();
    let mut graph = adj.to_vec();
    loop {
        let moved = local_moves(&graph);
        let labels = connected_parts(&graph, &moved);
        let count = labels
            .iter()
            .copied()
            .max()
            .map_or(0, |m| m.saturating_add(1));
        if count >= graph.len() {
            break;
        }
        for a in &mut assignment {
            *a = labels.get(*a).copied().unwrap_or(*a);
        }
        graph = aggregate(&graph, &labels, count);
    }
    assignment
}

// needed helper: one Louvain level of greedy modularity moves
fn local_moves(graph: &[Vec<(usize, f64)>]) -> Vec<usize> {
    let n = graph.len();
    let k: Vec<f64> = graph
        .iter()
        .map(|l| l.iter().map(|&(_, w)| w).sum())
        .collect();
    let m2: f64 = k.iter().sum();
    let mut comm: Vec<usize> = (0..n).collect();
    if m2 <= 0.0 {
        return comm;
    }
    let mut tot = k.clone();
    let mut weight_to = vec![0.0f64; n];
    let mut touched: Vec<usize> = Vec::new();
    for _ in 0..MAX_PASSES {
        let mut moved = false;
        for (i, list) in graph.iter().enumerate() {
            let own = comm.get(i).copied().unwrap_or(i);
            let ki = k.get(i).copied().unwrap_or(0.0);
            touched.clear();
            for &(j, w) in list {
                if j == i {
                    continue;
                }
                let c = comm.get(j).copied().unwrap_or(j);
                if let Some(slot) = weight_to.get_mut(c) {
                    *slot += w;
                    touched.push(c);
                }
            }
            touched.sort_unstable();
            touched.dedup();
            if let Some(t) = tot.get_mut(own) {
                *t -= ki;
            }
            let gain = |c: usize| {
                weight_to.get(c).copied().unwrap_or(0.0)
                    - RESOLUTION * tot.get(c).copied().unwrap_or(0.0) * ki / m2
            };
            let mut best = own;
            let mut best_gain = gain(own);
            for &c in &touched {
                let g = gain(c);
                if g > best_gain + EPSILON {
                    best = c;
                    best_gain = g;
                }
            }
            if let Some(t) = tot.get_mut(best) {
                *t += ki;
            }
            if let Some(slot) = comm.get_mut(i) {
                *slot = best;
            }
            moved |= best != own;
            for &c in &touched {
                if let Some(slot) = weight_to.get_mut(c) {
                    *slot = 0.0;
                }
            }
        }
        if !moved {
            break;
        }
    }
    comm
}

// needed helper: refinement step, split communities into connected parts (dense labels)
fn connected_parts(graph: &[Vec<(usize, f64)>], comm: &[usize]) -> Vec<usize> {
    let mut labels: Vec<Option<usize>> = vec![None; graph.len()];
    let mut next = 0usize;
    for start in 0..graph.len() {
        if labels.get(start).copied().flatten().is_some() {
            continue;
        }
        let c = comm.get(start).copied();
        let mut stack = vec![start];
        if let Some(slot) = labels.get_mut(start) {
            *slot = Some(next);
        }
        while let Some(v) = stack.pop() {
            for &(u, _) in graph.get(v).map_or(&[][..], |l| l.as_slice()) {
                let unlabeled = labels.get(u).is_some_and(|l| l.is_none());
                if unlabeled && comm.get(u).copied() == c {
                    if let Some(slot) = labels.get_mut(u) {
                        *slot = Some(next);
                    }
                    stack.push(u);
                }
            }
        }
        next = next.saturating_add(1);
    }
    labels.into_iter().map(|l| l.unwrap_or(0)).collect()
}

// needed helper: collapse each community into one vertex, summing weights (self loops kept)
fn aggregate(
    graph: &[Vec<(usize, f64)>],
    labels: &[usize],
    count: usize,
) -> Vec<Vec<(usize, f64)>> {
    let mut sums: Vec<BTreeMap<usize, f64>> = vec![BTreeMap::new(); count];
    for (i, list) in graph.iter().enumerate() {
        let Some(&ci) = labels.get(i) else {
            continue;
        };
        let Some(row) = sums.get_mut(ci) else {
            continue;
        };
        for &(j, w) in list {
            if let Some(&cj) = labels.get(j) {
                *row.entry(cj).or_insert(0.0) += w;
            }
        }
    }
    sums.into_iter()
        .map(|row| row.into_iter().collect())
        .collect()
}

// needed helper: map persona communities back to nodes, drop hubs and tiny groups
fn collect_groups(
    node_count: usize,
    owner: &[usize],
    community: &[usize],
) -> Vec<index::ItemGroup> {
    let mut memberships: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); node_count];
    for (persona, &node) in owner.iter().enumerate() {
        let Some(&c) = community.get(persona) else {
            continue;
        };
        if let Some(set) = memberships.get_mut(node) {
            set.insert(c);
        }
    }
    let mut members: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (node, set) in memberships.iter().enumerate() {
        if set.len() > MAX_GROUPS_PER_NODE {
            continue;
        }
        for &c in set {
            members.entry(c).or_default().push(node);
        }
    }
    let merged = merge_overlapping(members.into_values().map(BTreeSet::from_iter).collect());
    let mut groups: Vec<Vec<usize>> = merged
        .into_iter()
        .filter(|m| m.len() >= MIN_GROUP_SIZE)
        .map(|m| m.into_iter().collect())
        .collect();
    groups.sort_by(|a, b| {
        b.len()
            .cmp(&a.len())
            .then_with(|| a.first().cmp(&b.first()))
    });
    groups.into_iter().map(index::ItemGroup).collect()
}

// needed helper: fold together groups whose member sets mostly coincide (jaccard >= 1/2)
fn merge_overlapping(mut groups: Vec<BTreeSet<usize>>) -> Vec<BTreeSet<usize>> {
    loop {
        let mut pair = None;
        'search: for (i, a) in groups.iter().enumerate() {
            for (j, b) in groups.iter().enumerate().skip(i.saturating_add(1)) {
                let inter = a.intersection(b).count();
                let union = a.len().saturating_add(b.len()).saturating_sub(inter);
                if inter > 0 && inter.saturating_mul(2) >= union {
                    pair = Some((i, j));
                    break 'search;
                }
            }
        }
        let Some((i, j)) = pair else {
            return groups;
        };
        let taken = groups.remove(j);
        if let Some(target) = groups.get_mut(i) {
            target.extend(taken);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::detect_groups;

    #[test]
    fn test_usage() {
        let mut edges = Vec::new();
        for clique in [[0, 1, 2, 3], [4, 5, 6, 7]] {
            for (i, &a) in clique.iter().enumerate() {
                for &b in clique.iter().skip(i + 1) {
                    edges.push((a, b));
                }
            }
        }
        for n in [0, 1, 4, 5] {
            edges.push((8, n));
        }
        let groups = detect_groups(10, &edges);
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().all(|g| g.contains(&8)));
        let with_zero = groups.iter().find(|g| g.contains(&0)).unwrap();
        assert!(!with_zero.contains(&4));
        assert!(groups.iter().all(|g| !g.contains(&9)));
    }

    #[test]
    fn test_detect_groups_merges_near_duplicates() {
        let merged = super::merge_overlapping(vec![
            [1, 2, 3, 4].into_iter().collect(),
            [1, 2, 3, 5].into_iter().collect(),
            [7, 8, 9].into_iter().collect(),
        ]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged.first().map(|g| g.len()), Some(5));
    }

    #[test]
    fn test_detect_groups_is_deterministic() {
        let edges = [(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)];
        let a: Vec<Vec<usize>> = detect_groups(6, &edges)
            .iter()
            .map(|g| g.to_vec())
            .collect();
        let b: Vec<Vec<usize>> = detect_groups(6, &edges)
            .iter()
            .map(|g| g.to_vec())
            .collect();
        assert_eq!(a, b);
    }
}
