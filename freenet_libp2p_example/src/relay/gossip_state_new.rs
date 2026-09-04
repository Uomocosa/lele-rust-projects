use std::collections::HashMap;

use crate::relay;

#[must_use]
pub fn new() -> relay::gossip_state::GossipState {
    relay::gossip_state::GossipState {
        seen: HashMap::new(),
        last_next: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn test_usage() {
        let s = new();
        assert!(s.seen.is_empty());
    }
}
