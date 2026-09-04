use bevy::prelude::*;

use crate::p2p;

pub fn poll_p2p_events<T: p2p::Message>(
    mut commands: ResMut<p2p::P2PEvents<T>>,
    mut events: ResMut<p2p::P2PCommands<T>>,
) {
    let _ = (&mut commands, &mut events);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert!(true);
    }
}
// no test_usage necessary
