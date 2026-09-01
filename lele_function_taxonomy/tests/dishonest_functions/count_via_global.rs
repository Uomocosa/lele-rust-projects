use std::collections::BTreeMap;
use std::sync::Mutex;

static GLOBAL_SLOTS: Mutex<BTreeMap<u64, u64>> = Mutex::new(BTreeMap::new());

pub fn count_via_global() -> u64 {
    GLOBAL_SLOTS.lock().unwrap().values().sum()
}
