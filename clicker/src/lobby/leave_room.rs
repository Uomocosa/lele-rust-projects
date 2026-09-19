use crate::clicker;
use crate::lobby;

pub fn leave_room(
    lobby: &mut clicker::ActiveLobby,
    global: &mut clicker::GlobalCounter,
    pending: &mut clicker::PendingClicks,
    tombstones: &mut clicker::ScoreTombstones,
    selected: &mut lobby::SelectedRoom,
    left: &mut lobby::LeftRoom,
) {
    if !lobby.0.is_empty() {
        **left = Some(lobby.0.clone());
    }
    *lobby = clicker::ActiveLobby::default();
    *global = clicker::GlobalCounter::default();
    *pending = clicker::PendingClicks::default();
    tombstones.clear();
    **selected = None;
}

#[cfg(test)]
mod tests {
    use super::leave_room;
    use crate::clicker;
    use crate::lobby;

    #[test]
    fn test_usage() {
        let mut lobby = clicker::ActiveLobby("room-a".to_string());
        let mut global = clicker::GlobalCounter(42);
        let mut pending = clicker::PendingClicks {
            items: Vec::new(),
            parked: 3,
        };
        let mut tombstones = clicker::ScoreTombstones::default();
        tombstones.keep(1, 10);
        let mut selected = lobby::SelectedRoom(Some("room-a".to_string()));
        let mut left = lobby::LeftRoom::default();
        leave_room(
            &mut lobby,
            &mut global,
            &mut pending,
            &mut tombstones,
            &mut selected,
            &mut left,
        );
        assert_eq!(lobby, clicker::ActiveLobby::default());
        assert_eq!(*global, 0);
        assert_eq!(pending.parked, 0);
        assert!(tombstones.is_empty());
        assert!(selected.is_none());
        assert_eq!(*left, Some("room-a".to_string()));
    }

    #[test]
    fn leave_sticks_without_room_feed() {
        let mut lobby = clicker::ActiveLobby("room-a".to_string());
        let mut global = clicker::GlobalCounter(42);
        let mut pending = clicker::PendingClicks::default();
        let mut tombstones = clicker::ScoreTombstones::default();
        let mut selected = lobby::SelectedRoom(Some("room-a".to_string()));
        let mut left = lobby::LeftRoom::default();
        leave_room(
            &mut lobby,
            &mut global,
            &mut pending,
            &mut tombstones,
            &mut selected,
            &mut left,
        );
        assert!(selected.is_none());
        assert_eq!(lobby, clicker::ActiveLobby::default());
    }
}
