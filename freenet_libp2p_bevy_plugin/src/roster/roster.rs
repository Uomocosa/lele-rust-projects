use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use super::roster_add_entry;
use super::roster_remove_entry;

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut, Serialize, Deserialize)]
pub struct Roster(pub BTreeMap<String, BTreeMap<[u8; 32], String>>);

#[rustfmt::skip]
impl Roster {
    pub fn add_entry(&mut self, lobby: String, id: [u8; 32], addr: String) { roster_add_entry::add_entry(self, lobby, id, addr) }
    pub fn remove_entry(&mut self, lobby: &str, id: [u8; 32]) -> bool { roster_remove_entry::remove_entry(self, lobby, id) }
}

#[cfg(test)]
mod tests {
    use super::Roster;

    #[test]
    fn test_usage() {
        let mut r = Roster::default();
        r.add_entry("lobby".to_string(), [1u8; 32], "addr".to_string());
        assert_eq!(r.len(), 1);
    }
}
