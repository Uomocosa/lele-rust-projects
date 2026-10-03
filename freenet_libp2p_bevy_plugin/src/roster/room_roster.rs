use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut, Serialize, Deserialize)]
pub struct RoomRoster(pub BTreeMap<String, BTreeMap<[u8; 32], String>>);

#[rustfmt::skip]
impl RoomRoster {
    pub fn add_entry(&mut self, room: String, id: [u8; 32], addr: String) {
        self.entry(room).or_default().insert(id, addr);
    }

    pub fn remove_entry(&mut self, room: &str, id: [u8; 32]) -> bool {
        self.get_mut(room)
            .is_some_and(|members| members.remove(&id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::RoomRoster;

    #[test]
    fn test_usage() {
        let mut r = RoomRoster::default();
        r.add_entry("room".to_string(), [1u8; 32], "addr".to_string());
        assert_eq!(r.len(), 1);
    }
}
