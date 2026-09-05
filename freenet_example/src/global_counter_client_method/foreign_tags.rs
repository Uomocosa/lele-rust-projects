use crate::global_counter_client;
use crate::global_counter_client::GlobalCounterClient;
use global_counter_client::Pubkey;
use std::collections::BTreeMap;

#[must_use]
pub fn foreign_tags(client: &GlobalCounterClient) -> Vec<Pubkey> {
    foreign_in(&client.slots, client.pubkey)
}

// needed helper:
fn foreign_in(slots: &BTreeMap<Pubkey, u64>, pubkey: Pubkey) -> Vec<Pubkey> {
    slots.keys().copied().filter(|t| *t != pubkey).collect()
}

#[cfg(test)]
mod tests {
    use super::foreign_in;
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
        assert_eq!(foreign_in(&slots, pk(0)), vec![pk(1), pk(2)]);
        assert_eq!(foreign_in(&slots, pk(1)), vec![pk(0), pk(2)]);
        let own_only = BTreeMap::from([(pk(0), 5u64)]);
        assert_eq!(
            foreign_in(&own_only, pk(0)),
            Vec::<global_counter_client::Pubkey>::new()
        );
    }
}
