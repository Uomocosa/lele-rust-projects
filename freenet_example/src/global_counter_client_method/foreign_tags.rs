use crate::global_counter_client;
use std::collections::BTreeMap;

#[must_use]
pub fn foreign_tags(client: &global_counter_client::GlobalCounterClient) -> Vec<u64> {
    foreign_in(&client.slots, client.tag)
}

// needed helper:
fn foreign_in(slots: &BTreeMap<u64, u64>, tag: u64) -> Vec<u64> {
    slots.keys().copied().filter(|t| *t != tag).collect()
}

#[cfg(test)]
mod tests {
    use super::foreign_in;
    use std::collections::BTreeMap;

    #[test]
    fn test_usage() {
        let slots = BTreeMap::from([(0u64, 5u64), (1, 3), (2, 7)]);
        assert_eq!(foreign_in(&slots, 0), vec![1, 2]);
        assert_eq!(foreign_in(&slots, 1), vec![0, 2]);
        let own_only = BTreeMap::from([(0u64, 5u64)]);
        assert!(foreign_in(&own_only, 0).is_empty());
    }
}
