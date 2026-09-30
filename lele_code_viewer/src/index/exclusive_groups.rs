use std::collections::BTreeMap;

use crate::index;

const MIN_GROUP_SIZE: usize = 3;

pub fn exclusive_groups(
    groups: &[index::ItemGroup],
    edges: &[(usize, usize)],
) -> Vec<index::ItemGroup> {
    let mut memberships: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (g, group) in groups.iter().enumerate() {
        for &node in group.iter() {
            memberships.entry(node).or_default().push(g);
        }
    }
    let mut members: Vec<Vec<usize>> = vec![Vec::new(); groups.len()];
    for (&node, candidates) in &memberships {
        let best = candidates
            .iter()
            .copied()
            .max_by_key(|&g| (links_into(node, groups.get(g), edges), std::cmp::Reverse(g)));
        if let Some(list) = best.and_then(|g| members.get_mut(g)) {
            list.push(node);
        }
    }
    members
        .into_iter()
        .filter(|m| m.len() >= MIN_GROUP_SIZE)
        .map(index::ItemGroup)
        .collect()
}

// needed helper: number of edges between a node and the other members of a group
fn links_into(node: usize, group: Option<&index::ItemGroup>, edges: &[(usize, usize)]) -> usize {
    let Some(group) = group else {
        return 0;
    };
    edges
        .iter()
        .filter(|&&(a, b)| (a == node && group.contains(&b)) || (b == node && group.contains(&a)))
        .count()
}

#[cfg(test)]
mod tests {
    use super::exclusive_groups;
    use crate::index;

    #[test]
    fn test_usage() {
        let groups = vec![
            index::ItemGroup(vec![0, 1, 2, 6]),
            index::ItemGroup(vec![3, 4, 5, 6]),
        ];
        let edges = [(6, 3), (6, 4), (6, 0)];
        let single = exclusive_groups(&groups, &edges);
        assert_eq!(single.len(), 2);
        let first = single.first().unwrap();
        let second = single.get(1).unwrap();
        assert!(!first.contains(&6));
        assert!(second.contains(&6));
    }
}
