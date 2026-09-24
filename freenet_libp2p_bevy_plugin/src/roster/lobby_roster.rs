use std::collections::BTreeMap;

use atomic_delegate_macros::atomic_delegate;
use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut, Serialize, Deserialize)]
pub struct LobbyRoster(pub BTreeMap<String, BTreeMap<[u8; 32], String>>);

#[atomic_delegate]
impl LobbyRoster {
    pub fn add_entry(&mut self, lobby: String, id: [u8; 32], addr: String) {}
    pub fn remove_entry(&mut self, lobby: &str, id: [u8; 32]) -> bool {}
}

#[cfg(test)]
mod tests {
    use super::LobbyRoster;

    #[test]
    fn test_usage() {
        let mut r = LobbyRoster::default();
        r.add_entry("lobby".to_string(), [1u8; 32], "addr".to_string());
        assert_eq!(r.len(), 1);
    }
}
