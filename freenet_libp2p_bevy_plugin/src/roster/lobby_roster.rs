use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut, Serialize, Deserialize)]
pub struct LobbyRoster(pub BTreeMap<String, BTreeMap<[u8; 32], String>>);

#[rustfmt::skip]
impl LobbyRoster {
    pub fn add_entry(&mut self, lobby: String, id: [u8; 32], addr: String) {
        self.entry(lobby).or_default().insert(id, addr);
    }

    pub fn remove_entry(&mut self, lobby: &str, id: [u8; 32]) -> bool {
        self.get_mut(lobby)
            .is_some_and(|members| members.remove(&id).is_some())
    }
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
