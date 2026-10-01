use std::time::{Duration, Instant};

use crate::discovery;
use crate::p2p;
use discovery::id::{RemotePeerId, now_epoch};
use discovery::link::NetLink;
use discovery::room_peers::DiscoveryStatus;
use discovery::session::{Session, absorb_hello, dial_candidates, send_hello};
use discovery::{Event, Timing};

pub fn apply_tap(
    session: &mut Session,
    link: &NetLink,
    event: p2p::TapEvent,
    timing: &Timing,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    match event {
        p2p::TapEvent::PeerConnected(peer) => {
            let peer = RemotePeerId(peer);
            session.connected.insert(peer.clone());
            send_hello(session, link, &peer);
        }
        p2p::TapEvent::PeerDisconnected(peer) => {
            let peer = RemotePeerId(peer);
            session.connected.remove(&peer);
            mark_lost(session, &peer, events);
        }
        p2p::TapEvent::Exchange { from, data } => {
            absorb_hello(
                session,
                link,
                &RemotePeerId(from),
                &data,
                events,
                now_epoch(),
            );
        }
        _ => return,
    }
    let redial = Duration::from_secs(timing.redial_secs);
    dial_candidates(session, link, redial, Instant::now());
}

// needed helper: keeps a dropped member for the grace window, marked not connected
fn mark_lost(
    session: &mut Session,
    peer: &RemotePeerId,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    if let Some(room) = session.room.as_mut()
        && let Some(member) = room.members.get_mut(peer)
    {
        member.status = DiscoveryStatus::Known;
        member.presence.updated_at = now_epoch();
        let _ = events.send(Event::MembersChanged);
    }
}

// no test_usage necessary
