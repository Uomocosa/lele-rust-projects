use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref)]
pub struct RemotePeerId(pub String);

#[cfg(test)]
mod tests {
    use super::RemotePeerId;

    #[test]
    fn test_usage() {
        let peer = RemotePeerId("12D3KooWpeer".to_string());
        assert_eq!(peer.as_str(), "12D3KooWpeer");
        assert!(!peer.is_empty());
    }
}
