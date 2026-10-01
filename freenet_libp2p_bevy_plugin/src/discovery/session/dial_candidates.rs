use std::time::{Duration, Instant};

use crate::discovery;
use crate::p2p;
use discovery::link::NetLink;
use discovery::session::Session;

pub fn dial_candidates(session: &mut Session, link: &NetLink, redial: Duration, now: Instant) {
    if session.room.is_none() {
        return;
    }
    for (peer, presence) in &session.candidates {
        if session.connected.contains(peer) {
            continue;
        }
        let due = session
            .last_dial
            .get(peer)
            .is_none_or(|last| now.saturating_duration_since(*last) >= redial);
        if !due {
            continue;
        }
        session.last_dial.insert(peer.clone(), now);
        tracing::debug!(target: "room_lobby", peer = %peer.as_str(), "discovery dial candidate");
        link.tx
            .send(p2p::NetCommand::Dial {
                peer_id: (**peer).clone(),
                addrs: presence.addrs.clone(),
            })
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::dial_candidates;
    use crate::discovery;
    use crate::p2p;
    use discovery::id::{EpochSecs, RemotePeerId};

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (_, observed) = tokio::sync::watch::channel(None);
        let link = discovery::link::NetLink { tx, observed };
        let mut session = discovery::session::Session::new(
            discovery::id::UniqueGameId::new(
                &discovery::id::GameName("g".to_string()),
                &discovery::id::GameToken("t".to_string()),
            ),
            RemotePeerId("me".to_string()),
            Vec::new(),
            8,
        );
        session.room = Some(discovery::session::Room {
            name: discovery::id::RoomName("r".to_string()),
            members: discovery::room_peers::Members::new(),
        });
        let peers = vec![
            (RemotePeerId("a".to_string()), vec!["/ip4/1".to_string()]),
            (RemotePeerId("b".to_string()), vec!["/ip4/2".to_string()]),
        ];
        discovery::session::add_candidates(&mut session, peers, EpochSecs(1));
        session.connected.insert(RemotePeerId("b".to_string()));
        let now = Instant::now();
        dial_candidates(&mut session, &link, Duration::from_secs(2), now);
        dial_candidates(&mut session, &link, Duration::from_secs(2), now);
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::NetCommand::Dial { peer_id, .. }) if peer_id == "a"
        ));
        assert!(rx.try_recv().is_err());
    }
}
