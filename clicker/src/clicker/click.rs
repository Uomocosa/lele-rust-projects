use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::{p2p, roster};

use crate::clicker;

pub fn click(
    counter: &mut clicker::ClickCounter,
    global: &mut clicker::GlobalCounter,
    out: &mut p2p::Commands<clicker::ClickDelta>,
    roster: &roster::Roster,
    lobby: &clicker::ActiveLobby,
    own: net_id::NetworkId,
) {
    counter.increment();
    global.increment();
    let delta = clicker::ClickDelta {
        owner: own,
        delta: 1,
    };
    if let Some(members) = roster.get(&**lobby) {
        for peer in members.values() {
            out.push(p2p::Command::Send {
                peer_id: peer.clone(),
                payload: delta,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::click;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    #[test]
    fn test_usage() {
        let mut counter = clicker::ClickCounter::default();
        let mut global = clicker::GlobalCounter::default();
        let mut out = p2p::Commands::<clicker::ClickDelta>::default();
        let roster = roster::Roster::default();
        let lobby = clicker::ActiveLobby("alpha".to_string());
        click(
            &mut counter,
            &mut global,
            &mut out,
            &roster,
            &lobby,
            net_id::NetworkId(1),
        );
        assert_eq!(*counter, 1);
        assert_eq!(*global, 1);
        assert!(out.is_empty());
    }
}
