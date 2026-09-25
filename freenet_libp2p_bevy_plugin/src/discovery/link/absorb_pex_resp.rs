use super::super::constants;
use super::super::directory::Entry;
use super::super::gossip::peer_hint::PeerHint;
use super::dial_hint::dial_hint;
use super::run_context::RunContext;
use super::send_merged_directory::send_merged_directory;

pub fn absorb_pex_resp(
    ctx: &mut RunContext,
    _from: &str,
    peers: &[PeerHint],
    rooms: &[(String, Entry)],
) {
    for hint in peers.iter().take(constants::PEX_MAX_HINTS) {
        if hint.peer_id == ctx.peer_id {
            continue;
        }
        ctx.pex.insert(hint.clone());
        dial_hint(ctx, hint);
    }
    for (name, entry) in rooms.iter().take(constants::PEX_MAX_ROOMS) {
        if name.is_empty() || entry.params.is_empty() {
            continue;
        }
        let keep = ctx
            .known_rooms
            .get(name)
            .is_none_or(|known| entry.updated_at >= known.updated_at);
        if keep {
            if !ctx.known_rooms.contains_key(name)
                && ctx.known_rooms.len() >= constants::PEX_MAX_ROOMS
            {
                continue;
            }
            ctx.known_rooms.insert(name.clone(), entry.clone());
        }
    }
    send_merged_directory(ctx);
}

// no test_usage necessary
