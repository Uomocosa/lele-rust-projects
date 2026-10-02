use crate::discovery;
use crate::p2p;
use discovery::session::{Output, Session};

pub fn dial_candidates(session: &mut Session, now: discovery::EpochSecs) {
    if session.room.is_none() {
        return;
    }
    let redial_secs = session.timing.redial_secs;
    let due: Vec<(discovery::PeerId, Vec<String>)> = session
        .candidates
        .iter()
        .filter(|(peer, _)| !session.connected.contains(*peer))
        .filter(|(peer, _)| {
            session
                .last_dial
                .get(*peer)
                .is_none_or(|last| now.saturating_sub(**last) >= redial_secs)
        })
        .map(|(peer, presence)| (peer.clone(), presence.addrs.clone()))
        .collect();
    for (peer, addrs) in due {
        tracing::debug!(target: "room_lobby", peer = %peer.as_str(), "discovery dial candidate");
        session.last_dial.insert(peer.clone(), now);
        session.outputs.push(Output::Net(p2p::NetCommand::Dial {
            peer_id: peer.to_string(),
            addrs,
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::dial_candidates;
    use crate::discovery;
    use crate::p2p;
    use discovery::session::{Output, Session};

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            discovery::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        session.room = Some(discovery::Room {
            name: discovery::RoomName("r".to_string()),
            members: discovery::Members::new(),
        });
        let peers = vec![
            (
                discovery::PeerId("a".to_string()),
                vec!["/ip4/1".to_string()],
            ),
            (
                discovery::PeerId("b".to_string()),
                vec!["/ip4/2".to_string()],
            ),
        ];
        discovery::session::add_candidates(&mut session, peers, discovery::EpochSecs(1));
        session.connected.insert(discovery::PeerId("b".to_string()));
        dial_candidates(&mut session, discovery::EpochSecs(10));
        dial_candidates(&mut session, discovery::EpochSecs(10));
        assert!(matches!(
            std::mem::take(&mut session.outputs).as_slice(),
            [Output::Net(p2p::NetCommand::Dial { peer_id, .. })] if peer_id == "a"
        ));
        dial_candidates(&mut session, discovery::EpochSecs(12));
        assert_eq!(std::mem::take(&mut session.outputs).len(), 1);
    }
}
