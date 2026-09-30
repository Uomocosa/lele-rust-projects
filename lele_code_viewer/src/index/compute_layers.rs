pub fn compute_layers(node_count: usize, user_to_dep: &[(usize, usize)]) -> Vec<usize> {
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    for &(user, dep) in user_to_dep {
        if let Some(list) = adjacency.get_mut(user) {
            list.push(dep);
        }
    }
    let mut levels = vec![0usize; node_count];
    let mut state = vec![0u8; node_count];
    for node in 0..node_count {
        visit(node, &adjacency, &mut levels, &mut state);
    }
    levels
}

// needed helper: depth-first longest path with cycle guard
fn visit(node: usize, adjacency: &[Vec<usize>], levels: &mut [usize], state: &mut [u8]) {
    if state.get(node).copied().unwrap_or(2) != 0 {
        return;
    }
    if let Some(slot) = state.get_mut(node) {
        *slot = 1;
    }
    let mut best = 0usize;
    if let Some(deps) = adjacency.get(node).cloned() {
        for dep in deps {
            if state.get(dep).copied().unwrap_or(2) == 1 {
                continue;
            }
            visit(dep, adjacency, levels, state);
            let candidate = levels.get(dep).copied().unwrap_or(0).saturating_add(1);
            best = best.max(candidate);
        }
    }
    if let Some(slot) = levels.get_mut(node) {
        *slot = best;
    }
    if let Some(slot) = state.get_mut(node) {
        *slot = 2;
    }
}

#[cfg(test)]
mod tests {
    use super::compute_layers;

    #[test]
    fn test_usage() {
        let levels = compute_layers(3, &[(2, 1), (1, 0)]);
        assert_eq!(levels, vec![0, 1, 2]);
        let cycled = compute_layers(2, &[(0, 1), (1, 0)]);
        assert!(cycled.iter().all(|&l| l <= 1));
    }
}
