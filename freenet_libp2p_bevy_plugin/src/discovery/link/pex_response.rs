use super::super::constants;
use super::super::directory::Entry;
use super::super::gossip::merge_peer_hints::merge_peer_hints;
use super::super::gossip::peer_hint::PeerHint;
use super::epoch_secs::epoch_secs;
use super::pex_msg::PexMsg;
use super::run_context::RunContext;

#[must_use]
pub fn pex_response(ctx: &RunContext) -> PexMsg {
    let mut peers: Vec<PeerHint> = ctx
        .roster
        .slots
        .values()
        .map(|entry| PeerHint {
            peer_id: entry.peer_id.clone(),
            addrs: entry.addrs.clone(),
            rooms: Vec::new(),
            updated_at: entry.updated_at,
        })
        .collect();
    for hint in ctx.pex.values() {
        peers.push(hint.clone());
    }
    let peers = merge_peer_hints(peers)
        .into_iter()
        .take(constants::PEX_MAX_HINTS)
        .collect();
    let rooms: Vec<(String, Entry)> = ctx
        .known_rooms
        .iter()
        .map(|(name, entry)| (name.clone(), entry.clone()))
        .chain([(
            ctx.room.clone(),
            Entry {
                params: ctx.room_params.clone(),
                peer_id: ctx.peer_id.clone(),
                addrs: ctx.last_addrs.clone(),
                updated_at: epoch_secs(),
            },
        )])
        .take(constants::PEX_MAX_ROOMS)
        .collect();
    PexMsg::Resp { peers, rooms }
}

// no test_usage necessary
