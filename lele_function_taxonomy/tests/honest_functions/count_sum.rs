use std::collections::BTreeMap;

pub fn count_sum(slots: &BTreeMap<u64, u64>) -> u64 {
    slots.values().sum()
}
