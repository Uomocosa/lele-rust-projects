use super::active_room::ActiveRoom;
use super::join_gate::JoinGate;
use super::join_pending::JoinPending;
use super::left_room::LeftRoom;
use super::selected_room::SelectedRoom;

pub fn leave_rooms(
    active: &mut ActiveRoom,
    selected: &mut SelectedRoom,
    left: &mut LeftRoom,
    pending: &mut JoinPending,
    gate: &mut JoinGate,
) {
    if let Some(room) = active.as_ref() {
        **left = Some(room.clone());
    }
    **active = None;
    **selected = None;
    **pending = None;
    gate.expected = None;
    gate.committed = false;
}

#[cfg(test)]
mod tests {
    use super::{ActiveRoom, JoinGate, JoinPending, LeftRoom, SelectedRoom, leave_rooms};
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut active = ActiveRoom(Some(discovery::params::RoomName("room-a".to_string())));
        let mut selected = SelectedRoom(Some(discovery::params::RoomName("room-a".to_string())));
        let mut left = LeftRoom::default();
        let mut pending = JoinPending(Some(discovery::params::RoomName("room-a".to_string())));
        let mut gate = JoinGate::default();
        leave_rooms(
            &mut active,
            &mut selected,
            &mut left,
            &mut pending,
            &mut gate,
        );
        assert!(active.is_none());
        assert!(selected.is_none());
        assert!(left.as_ref().is_some_and(|room| room.as_str() == "room-a"));
        assert!(pending.is_none());
        assert!(gate.expected.is_none());
        assert!(!gate.committed);
    }
}
