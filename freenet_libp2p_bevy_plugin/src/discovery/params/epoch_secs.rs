use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref,
)]
pub struct EpochSecs(pub u64);

#[cfg(test)]
mod tests {
    use super::EpochSecs;

    #[test]
    fn test_usage() {
        let stamp = EpochSecs(7);
        assert_eq!(*stamp, 7);
        assert!(EpochSecs(9) > stamp);
    }
}
