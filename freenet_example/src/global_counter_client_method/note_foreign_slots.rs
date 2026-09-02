use crate::global_counter_client;
use std::collections::BTreeMap;
use std::time::Instant;

/// Tracks foreign-slot freshness.
///
/// `foreign_seen` only advances when the summed foreign values actually
/// change, so a stale subscription (foreign slots frozen while peers keep
/// ticking) still arms the bridge.
pub fn note_foreign_slots(client: &mut global_counter_client::GlobalCounterClient) {
    let sum = foreign_sum(&client.slots, client.tag);
    if sum != client.foreign_sum {
        client.foreign_sum = sum;
        client.foreign_seen = Some(Instant::now());
    }
}

// needed helper:
fn foreign_sum(slots: &BTreeMap<u64, u64>, tag: u64) -> u64 {
    slots
        .iter()
        .filter(|(t, _)| **t != tag)
        .map(|(_, v)| v)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::foreign_sum;
    use std::collections::BTreeMap;

    #[test]
    fn test_usage() {
        let slots = BTreeMap::from([(0u64, 5u64), (1, 3), (2, 7)]);
        assert_eq!(foreign_sum(&slots, 0), 10);
        assert_eq!(foreign_sum(&slots, 1), 12);
        let own_only = BTreeMap::from([(0u64, 5u64)]);
        assert_eq!(foreign_sum(&own_only, 0), 0);
    }
}
