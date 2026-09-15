use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;

pub fn click(
    counter: &mut clicker::ClickCounter,
    global: &mut clicker::GlobalCounter,
    out: &mut p2p::Commands<clicker::CursorMsg>,
    lobby: &clicker::ActiveLobby,
    own: net_id::NetworkId,
) {
    counter.increment();
    global.increment();
    let delta = clicker::CursorMsg::Click {
        owner: own,
        delta: 1,
    };
    let data = bincode::serialize(&delta).unwrap_or_default();
    out.push(p2p::Command::Publish {
        topic: clicker::click_topic(lobby),
        data,
    });
}

#[cfg(test)]
mod tests {
    use super::click;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    #[test]
    fn test_usage() {
        let mut counter = clicker::ClickCounter::default();
        let mut global = clicker::GlobalCounter::default();
        let mut out = p2p::Commands::<clicker::CursorMsg>::default();
        let lobby = clicker::ActiveLobby("alpha".to_string());
        click(
            &mut counter,
            &mut global,
            &mut out,
            &lobby,
            net_id::NetworkId(1),
        );
        assert_eq!(*counter, 1);
        assert_eq!(*global, 1);
        assert_eq!(out.len(), 1);
    }
}
