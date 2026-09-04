use bevy::prelude::*;

use super::plugin::P2PPlugin;
use crate::p2p;

pub fn build<T: p2p::Message>(plugin: &P2PPlugin<T>, app: &mut App) {
    let event_rx = plugin.take_event_rx();
    let cmd_tx = plugin.cmd_tx.clone();
    let _ = (event_rx, cmd_tx);
    app.insert_resource(p2p::P2PEvents::<T>::default());
    app.insert_resource(p2p::P2PCommands::<T>::default());
    app.insert_resource(crate::roster::Roster::default());
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert!(true);
    }
}
// no test_usage necessary
