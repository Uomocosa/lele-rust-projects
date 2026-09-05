use bevy::prelude::*;

use super::p2p_plugin::P2PPlugin;
use crate::p2p;
use crate::roster;

pub fn build<T: p2p::Message>(plugin: &P2PPlugin<T>, app: &mut App) {
    let event_rx = plugin.take_event_rx();
    let cmd_tx = plugin.cmd_tx.clone();
    let _ = (event_rx, cmd_tx);
    app.insert_resource(p2p::Events::<T>::default());
    app.insert_resource(p2p::Commands::<T>::default());
    app.insert_resource(roster::Roster::default());
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(build);
    }
}
// no test_usage necessary
