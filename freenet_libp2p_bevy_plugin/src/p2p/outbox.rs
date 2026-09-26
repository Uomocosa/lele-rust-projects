use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::p2p;
use p2p::NetCommand;

#[derive(Resource, Debug, Default, Deref, DerefMut, Serialize, Deserialize)]
pub struct Outbox(pub Vec<NetCommand>);

#[atomic_delegates]
impl Outbox {
    pub fn take_all(&mut self) -> Vec<NetCommand> {}
}

#[cfg(test)]
mod tests {
    use super::Outbox;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut commands = Outbox::default();
        assert!(commands.is_empty());
        commands.push(p2p::NetCommand::FindLobby {
            lobby: "lobby".to_string(),
        });
        assert_eq!(commands.len(), 1);
    }
}
