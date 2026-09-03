use crate::global_counter_client;
use std::collections::BTreeMap;
use std::time::Instant;

pub fn note_foreign_slots_at(
    client: &mut global_counter_client::GlobalCounterClient,
    now: Instant,
) {
    let sum = foreign_sum(&client.slots, client.pubkey);
    if sum != client.foreign_sum {
        client.foreign_sum = sum;
        client.foreign_seen = Some(now);
    }
}

// needed helper:
fn foreign_sum(
    slots: &BTreeMap<global_counter_client::Pubkey, u64>,
    pubkey: global_counter_client::Pubkey,
) -> u64 {
    slots
        .iter()
        .filter(|(t, _)| **t != pubkey)
        .map(|(_, v)| v)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::foreign_sum;
    use crate::global_counter_client;
    use std::collections::BTreeMap;

    fn pk(n: u8) -> global_counter_client::Pubkey {
        let mut p = [0u8; 32];
        p[0] = n;
        p
    }

    #[test]
    fn test_usage() {
        let slots = BTreeMap::from([(pk(0), 5u64), (pk(1), 3), (pk(2), 7)]);
        assert_eq!(foreign_sum(&slots, pk(0)), 10);
    }
}
