use bevy::prelude::*;

use crate::net_id;
use crate::p2p;
use crate::roster;

pub fn dial_roster_peers<T: p2p::Message>(
    commands: ResMut<p2p::P2pCommands<T>>,
    roster: Res<roster::Roster>,
    mut dialed: ResMut<p2p::DialedPeers>,
    identity: Res<net_id::LocalIdentity>,
) {
    if !roster.is_changed() {
        return;
    }
    for (id, entry) in roster.iter() {
        if *id == **identity || dialed.contains(&entry.peer_id) {
            continue;
        }
        commands
            .send(p2p::Command::Dial {
                peer_id: entry.peer_id.clone(),
                addrs: entry.addrs.clone(),
            })
            .ok();
        dialed.insert(entry.peer_id.clone());
    }
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use bevy::prelude::*;

    use super::dial_roster_peers;
    use crate::net_id;
    use crate::p2p;
    use crate::roster;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Command<Dummy>>();
        let mut app = App::new();
        app.insert_resource(net_id::LocalIdentity(net_id::NetworkId(1)));
        let mut entries = roster::RosterState::new();
        entries.insert(
            net_id::NetworkId(1),
            roster::PeerEntry {
                peer_id: "self".to_string(),
                addrs: vec![],
                updated_at: 1,
            },
        );
        entries.insert(
            net_id::NetworkId(2),
            roster::PeerEntry {
                peer_id: "peer-2".to_string(),
                addrs: vec!["/ip4/127.0.0.1/tcp/4000".to_string()],
                updated_at: 1,
            },
        );
        app.insert_resource(roster::Roster(entries));
        app.insert_resource(p2p::DialedPeers::default());
        app.insert_resource(p2p::P2pCommands(cmd_tx));
        app.add_systems(Update, dial_roster_peers::<Dummy>);

        app.update();

        assert!(matches!(
            cmd_rx.try_recv(),
            Ok(p2p::Command::Dial { peer_id, .. }) if peer_id == "peer-2"
        ));
        assert!(!matches!(
            cmd_rx.try_recv(),
            Ok(p2p::Command::Dial { peer_id, .. }) if peer_id == "self"
        ));
    }
}
