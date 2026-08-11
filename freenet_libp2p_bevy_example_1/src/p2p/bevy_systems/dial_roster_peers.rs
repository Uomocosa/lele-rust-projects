use bevy::prelude::*;

use crate::boxes;
use crate::p2p;
use crate::roster;

pub fn dial_roster_peers(
    commands: ResMut<p2p::P2pCommands>,
    roster: Res<roster::Roster>,
    mut dialed: ResMut<p2p::DialedPeers>,
    config: Res<boxes::Config>,
) {
    if !roster.is_changed() {
        return;
    }
    for (player_id, entry) in roster.iter() {
        if *player_id == **config || dialed.contains(&entry.peer_id) {
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
    use bevy::prelude::*;

    use super::dial_roster_peers;
    use crate::boxes;
    use crate::p2p;
    use crate::roster;

    #[test]
    fn test_usage() {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Command>();
        let mut app = App::new();
        app.insert_resource(boxes::Config::new(boxes::PlayerId(1)));
        let mut entries = roster::RosterState::new();
        entries.insert(
            boxes::PlayerId(2),
            roster::PeerEntry {
                peer_id: "peer-2".to_string(),
                addrs: vec!["/ip4/127.0.0.1/tcp/4000".to_string()],
                updated_at: 1,
            },
        );
        app.insert_resource(roster::Roster(entries));
        app.insert_resource(p2p::DialedPeers::default());
        app.insert_resource(p2p::P2pCommands(cmd_tx));
        app.add_systems(Update, dial_roster_peers);

        app.update();

        assert!(matches!(
            cmd_rx.try_recv(),
            Ok(p2p::Command::Dial { peer_id, .. }) if peer_id == "peer-2"
        ));
    }
}
