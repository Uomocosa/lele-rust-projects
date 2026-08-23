use serde::{Deserialize, Serialize};

use crate::net_id;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot<T> {
    pub from_id: net_id::NetworkId,
    pub tick: u64,
    pub sent_at_ms: u64,
    pub payload: T,
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use super::Snapshot;
    use crate::net_id;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let snapshot = Snapshot {
            from_id: net_id::NetworkId(1),
            tick: 5,
            sent_at_ms: 100,
            payload: Dummy(7),
        };
        let encoded = bincode::serialize(&snapshot);
        let decoded = encoded.ok().and_then(|e| bincode::deserialize(&e).ok());
        assert_eq!(decoded, Some(snapshot));
    }
}
