use bevy::prelude::*;

use crate::p2p;

pub const fn poll_p2p_events<T: p2p::Message>(
    mut commands: ResMut<p2p::Events<T>>,
    mut events: ResMut<p2p::Commands<T>>,
) {
    let _ = (&mut commands, &mut events);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(poll_p2p_events);
    }
}
// no test_usage necessary
