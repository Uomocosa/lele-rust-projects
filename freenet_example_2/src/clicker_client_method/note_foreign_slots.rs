use crate::clicker_client;
use std::collections::BTreeMap;
use std::time::Instant;

pub fn note_foreign_slots(client: &mut clicker_client::ClickerClient) {
    refresh_foreign(&client.slots, client.tag, &mut client.foreign_seen);
}

// needed helper:
fn refresh_foreign(slots: &BTreeMap<u64, u64>, tag: u64, seen: &mut Option<Instant>) {
    if slots.keys().any(|t| *t != tag) {
        *seen = Some(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::refresh_foreign;
    use std::collections::BTreeMap;
    use std::time::Duration;

    #[test]
    fn test_usage() {
        let mut seen = None;
        let own_only = BTreeMap::from([(0u64, 5u64)]);
        refresh_foreign(&own_only, 0, &mut seen);
        assert!(seen.is_none());

        let mut with_foreign = own_only.clone();
        with_foreign.insert(1, 3);
        refresh_foreign(&with_foreign, 0, &mut seen);
        assert!(seen.is_some());

        let stamped = seen.unwrap();
        std::thread::sleep(Duration::from_millis(2));
        refresh_foreign(&own_only, 0, &mut seen);
        assert_eq!(seen.unwrap(), stamped);
    }
}
