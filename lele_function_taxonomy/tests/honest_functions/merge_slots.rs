use std::collections::BTreeMap;

pub fn merge_slots(slots: &mut BTreeMap<u64, u64>, incoming: BTreeMap<u64, u64>) {
    for (tag, value) in incoming {
        let entry = slots.entry(tag).or_insert(0);
        *entry = (*entry).max(value);
    }
}
