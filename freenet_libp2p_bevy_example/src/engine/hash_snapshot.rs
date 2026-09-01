use crate::engine;

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

pub fn hash_snapshot(snapshot: &engine::Snapshot) -> u64 {
    fnv1a(&canonical_bytes(snapshot))
}

// needed helper:
fn canonical_bytes(snapshot: &engine::Snapshot) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&snapshot.tick.to_le_bytes());
    let list: Vec<(engine::PlayerId, (f32, f32))> = snapshot
        .bodies
        .iter()
        .map(|(id, pos)| (*id, *pos))
        .collect();
    bytes.extend_from_slice(&(list.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&bincode::serialize(&list).unwrap_or_default());
    bytes
}

// needed helper:
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::hash_snapshot;
    use crate::engine::Snapshot;

    #[test]
    fn test_usage() {
        let mut bodies = BTreeMap::new();
        bodies.insert([1; 32], (1.0, 2.0));
        let a = Snapshot { tick: 1, bodies };
        let hash = hash_snapshot(&a);
        assert!(hash != 0);
        assert_eq!(hash, hash_snapshot(&a));
    }

    #[test]
    fn hash_ignores_map_insertion_order() {
        let mut bodies = BTreeMap::new();
        bodies.insert([1; 32], (1.0, 2.0));
        bodies.insert([2; 32], (3.0, 4.0));
        let a = Snapshot { tick: 1, bodies };

        let mut reversed = BTreeMap::new();
        reversed.insert([2; 32], (3.0, 4.0));
        reversed.insert([1; 32], (1.0, 2.0));
        let b = Snapshot {
            tick: 1,
            bodies: reversed,
        };

        assert_eq!(hash_snapshot(&a), hash_snapshot(&b));
    }
}
