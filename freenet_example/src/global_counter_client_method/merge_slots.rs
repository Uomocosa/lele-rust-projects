use std::collections::BTreeMap;

pub fn merge_slots(slots: &mut BTreeMap<u64, u64>, incoming: BTreeMap<u64, u64>) {
    for (tag, value) in incoming {
        let entry = slots.entry(tag).or_insert(0);
        *entry = (*entry).max(value);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::merge_slots;

    #[test]
    fn test_usage() {
        let mut slots = BTreeMap::from([(0u64, 5u64), (1, 2)]);
        merge_slots(&mut slots, BTreeMap::from([(1u64, 7u64), (2, 3)]));
        assert_eq!(slots.get(&0), Some(&5));
        assert_eq!(slots.get(&1), Some(&7));
        assert_eq!(slots.get(&2), Some(&3));
        merge_slots(&mut slots, BTreeMap::from([(1u64, 1u64)]));
        assert_eq!(slots.get(&1), Some(&7));
    }
}
