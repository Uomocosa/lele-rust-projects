use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Deref,
)]
pub struct PlayerId(pub u64);

#[cfg(test)]
mod tests {
    use super::PlayerId;

    #[test]
    fn test_usage() {
        let id = PlayerId(3);
        assert_eq!(*id, 3);
        let bytes = bincode::serialize(&id).unwrap_or_default();
        assert_eq!(bytes, 3u64.to_le_bytes());
    }
}
