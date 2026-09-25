use crate::roster;

use super::active_room::ActiveRoom;
use super::join_clock::JoinClock;
use super::join_gate::JoinGate;
use super::join_pending::JoinPending;
use super::selected_room::SelectedRoom;

pub fn confirm_joined(
    active: &mut ActiveRoom,
    selected: &mut SelectedRoom,
    pending: &mut JoinPending,
    gate: &mut JoinGate,
    clock: &mut JoinClock,
    lobby: &mut roster::Lobby,
    room: String,
) {
    **active = Some(room.clone());
    **selected = Some(room.clone());
    *lobby = roster::Lobby(room.clone());
    **pending = Some(room);
    gate.expected = None;
    gate.committed = false;
    clock.clicked_at = Some(std::time::Instant::now());
    clock.last_new_peer = None;
}

#[cfg(test)]
mod tests {
    use crate::roster;

    use super::{ActiveRoom, JoinClock, JoinGate, JoinPending, SelectedRoom, confirm_joined};

    #[test]
    fn test_usage() {
        let mut active = ActiveRoom::default();
        let mut selected = SelectedRoom::default();
        let mut pending = JoinPending::default();
        let mut gate = JoinGate::default();
        let mut clock = JoinClock::default();
        let mut lobby = roster::Lobby::default();
        confirm_joined(
            &mut active,
            &mut selected,
            &mut pending,
            &mut gate,
            &mut clock,
            &mut lobby,
            "room-a".to_string(),
        );
        assert_eq!(active.as_deref(), Some("room-a"));
        assert_eq!(selected.as_deref(), Some("room-a"));
        assert_eq!(pending.as_deref(), Some("room-a"));
        assert_eq!(lobby.as_str(), "room-a");
        assert!(gate.expected.is_none());
        assert!(!gate.committed);
        assert!(clock.clicked_at.is_some());
        assert!(clock.last_new_peer.is_none());
    }
}
