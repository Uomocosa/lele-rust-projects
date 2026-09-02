use crate::clicker_client;
use std::collections::BTreeMap;
use std::time::Instant;

pub fn note_foreign_slots_at(client: &mut clicker_client::ClickerClient, now: Instant) {
    let sum = foreign_sum(&client.slots, client.tag);
    if sum != client.foreign_sum {
        client.foreign_sum = sum;
        client.foreign_seen = Some(now);
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
    }
}
