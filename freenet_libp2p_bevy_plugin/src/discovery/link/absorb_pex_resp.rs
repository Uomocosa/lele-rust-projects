use super::super::constants;
use super::super::directory::room_payload::RoomPayload;
use super::super::gossip::peer_hint::PeerHint;
use super::super::params::room_name::RoomName;
use super::dial_hint::dial_hint;
use super::run_context::RunContext;
use super::send_merged_directory::send_merged_directory;

pub fn absorb_pex_resp(
    ctx: &mut RunContext,
    _from: &str,
    peers: &[PeerHint],
    rooms: &[(RoomName, RoomPayload)],
) {
    for hint in peers.iter().take(constants::PEX_MAX_HINTS) {
        if hint.peer_id == ctx.peer_id {
            continue;
        }
        ctx.pex.insert(hint.clone());
        dial_hint(ctx, hint);
    }
    for (name, payload) in rooms.iter().take(constants::PEX_MAX_ROOMS) {
        if name.is_empty() || payload.params.is_empty() {
            continue;
        }
        let keep = ctx
            .known_rooms
            .get(name)
            .is_none_or(|known| payload.updated_at >= known.updated_at);
        if keep {
            if !ctx.known_rooms.contains_key(name)
                && ctx.known_rooms.len() >= constants::PEX_MAX_ROOMS
            {
                continue;
            }
            ctx.known_rooms.insert(name.clone(), payload.clone());
        }
    }
    send_merged_directory(ctx);
}

// no test_usage necessary
