use crate::clicker;

pub fn leave_room(
    lobby: &mut clicker::ActiveLobby,
    global: &mut clicker::GlobalCounter,
    pending: &mut clicker::PendingClicks,
    tombstones: &mut clicker::ScoreTombstones,
) {
    *lobby = clicker::ActiveLobby::default();
    *global = clicker::GlobalCounter::default();
    *pending = clicker::PendingClicks::default();
    tombstones.clear();
}

#[cfg(test)]
mod tests {
    use super::leave_room;
    use crate::clicker;

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
        leave_room(&mut lobby, &mut global, &mut pending, &mut tombstones);
        assert_eq!(lobby, clicker::ActiveLobby::default());
        assert_eq!(*global, 0);
        assert_eq!(pending.parked, 0);
        assert!(tombstones.is_empty());
    }
}
