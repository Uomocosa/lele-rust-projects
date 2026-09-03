use std::collections::BTreeMap;

use crate::global_counter_client;
use global_counter_client::Pubkey;

pub fn merge_slots(slots: &mut BTreeMap<Pubkey, u64>, incoming: BTreeMap<Pubkey, u64>) {
    for (pubkey, value) in incoming {
        let entry = slots.entry(pubkey).or_insert(0);
        *entry = (*entry).max(value);
    }
}

#[cfg(test)]
mod tests {
    use crate::global_counter_client;
    use std::collections::BTreeMap;

    use super::merge_slots;

    fn pk(n: u8) -> global_counter_client::Pubkey {
        let mut p = [0u8; 32];
        p[0] = n;
        p
    }

    #[test]
    fn test_usage() {
        let mut slots = BTreeMap::from([(pk(0), 5u64), (pk(1), 2)]);
        merge_slots(&mut slots, BTreeMap::from([(pk(1), 7u64), (pk(2), 3)]));
        assert_eq!(slots.get(&pk(0)), Some(&5));
        assert_eq!(slots.get(&pk(1)), Some(&7));
        assert_eq!(slots.get(&pk(2)), Some(&3));
        merge_slots(&mut slots, BTreeMap::from([(pk(1), 1u64)]));
        assert_eq!(slots.get(&pk(1)), Some(&7));
    }
}
