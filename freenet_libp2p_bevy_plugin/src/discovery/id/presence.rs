use serde::{Deserialize, Serialize};

use super::epoch_secs::EpochSecs;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presence {
    pub addrs: Vec<String>,
    pub updated_at: EpochSecs,
}

#[cfg(test)]
mod tests {
    use super::Presence;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let presence = Presence {
            addrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
            updated_at: discovery::id::EpochSecs(7),
        };
        let bytes = bincode::serialize(&presence).unwrap_or_default();
        let decoded: Presence = bincode::deserialize(&bytes).expect("decodes");
        assert_eq!(decoded, presence);
    }
}
