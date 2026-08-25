use crate::engine;

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

pub fn hash_action(action: &engine::Action) -> u64 {
    let bytes = bincode::serialize(action).unwrap_or_default();
    fnv1a(&bytes)
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
    use crate::engine;

    use super::hash_action;

    #[test]
    fn test_usage() {
        let a = engine::Action {
            direction: engine::Direction::Right,
            jump: false,
        };
        let b = engine::Action {
            direction: engine::Direction::Left,
            jump: false,
        };
        assert_ne!(hash_action(&a), hash_action(&b));
        assert_eq!(hash_action(&a), hash_action(&a));
    }
}
