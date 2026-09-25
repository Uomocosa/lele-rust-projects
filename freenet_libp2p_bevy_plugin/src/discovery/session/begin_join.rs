use super::super::params::room_name::RoomName;
use super::join_pending::JoinPending;

#[must_use]
pub fn begin_join(pending: &mut JoinPending, room: RoomName) -> bool {
    if room.is_empty() || pending.is_some() {
        return false;
    }
    **pending = Some(room);
    true
}

#[cfg(test)]
mod tests {
    use super::{JoinPending, begin_join};
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut pending = JoinPending::default();
        assert!(!begin_join(
            &mut pending,
            discovery::params::RoomName(String::new())
        ));
        assert!(begin_join(
            &mut pending,
            discovery::params::RoomName("room-a".to_string())
        ));
        assert!(!begin_join(
            &mut pending,
            discovery::params::RoomName("room-b".to_string())
        ));
        assert!(
            pending
                .as_ref()
                .is_some_and(|room| room.as_str() == "room-a")
        );
    }
}
