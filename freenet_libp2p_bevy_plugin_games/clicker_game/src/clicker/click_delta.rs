use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Deref)]
pub struct ClickDelta(pub i32);

#[cfg(test)]
mod tests {
    use super::ClickDelta;

    #[test]
    fn test_usage() {
        let delta = ClickDelta(1);
        let encoded = bincode::serialize(&delta);
        let decoded = encoded.ok().and_then(|e| bincode::deserialize(&e).ok());
        assert_eq!(decoded, Some(delta));
    }
}
