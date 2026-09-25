use super::super::gossip::peer_hint::PeerHint;
use super::super::membership::roster_state::RosterState;
use super::dial_hint::dial_hint;
use super::run_context::RunContext;

pub fn absorb_roster_gossip(ctx: &mut RunContext, from: &str, data: &[u8]) {
    let slots: RosterState = bincode::deserialize(data).unwrap_or_default();
    for entry in slots.values() {
        if entry.peer_id.as_str() == from || entry.peer_id.is_empty() {
            continue;
        }
        let hint = PeerHint {
            peer_id: entry.peer_id.clone(),
            addrs: entry.addrs.clone(),
            rooms: Vec::new(),
            updated_at: entry.updated_at,
        };
        ctx.pex.insert(hint.clone());
        dial_hint(ctx, &hint);
    }
}

// no test_usage necessary
