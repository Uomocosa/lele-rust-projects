use std::collections::BTreeMap;

use crate::engine;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Snapshot {
    pub tick: u64,
    pub bodies: BTreeMap<engine::PlayerId, (f32, f32)>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::Snapshot;

    #[test]
    fn test_usage() {
        let mut bodies = BTreeMap::new();
        bodies.insert([1; 32], (1.0, 2.0));
        let snapshot = Snapshot { tick: 3, bodies };
        assert_eq!(snapshot.tick, 3);
        assert_eq!(snapshot.bodies.len(), 1);
    }
}
