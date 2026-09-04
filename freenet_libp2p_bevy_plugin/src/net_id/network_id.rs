use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Deref)]
pub struct NetworkId(pub u64);

#[cfg(test)]
mod tests {
    use super::NetworkId;

    #[test]
    fn test_usage() {
        let id = NetworkId(42);
        assert_eq!(*id, 42);
    }
}
