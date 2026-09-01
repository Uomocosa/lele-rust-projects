use std::collections::BTreeMap;

pub fn foreign_in(slots: &BTreeMap<u64, u64>, tag: u64) -> Vec<u64> {
    slots.keys().copied().filter(|t| *t != tag).collect()
}
