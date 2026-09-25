use std::collections::{HashMap, VecDeque};
use std::time::Instant;

pub(super) type ConnectedMap = HashMap<String, u32>;
pub(super) type StaggerMap = HashMap<String, (VecDeque<String>, Instant)>;
pub(super) type AttemptedMap = HashMap<String, Instant>;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{AttemptedMap, ConnectedMap, StaggerMap};

    #[test]
    fn test_usage() {
        let mut connected: ConnectedMap = HashMap::new();
        connected.insert("peer".to_string(), 1);
        assert_eq!(connected.get("peer"), Some(&1));
        let staggers: StaggerMap = HashMap::new();
        assert!(staggers.is_empty());
        let mut attempted: AttemptedMap = HashMap::new();
        attempted.insert("peer".to_string(), std::time::Instant::now());
        assert!(attempted.contains_key("peer"));
    }
}
