use super::join_pending::JoinPending;

#[must_use]
pub fn begin_join(pending: &mut JoinPending, room: String) -> bool {
    if room.is_empty() || pending.is_some() {
        return false;
    }
    **pending = Some(room);
    true
}

#[cfg(test)]
mod tests {
    use super::{JoinPending, begin_join};

    #[test]
    fn test_usage() {
        let mut pending = JoinPending::default();
        assert!(!begin_join(&mut pending, String::new()));
        assert!(begin_join(&mut pending, "room-a".to_string()));
        assert!(!begin_join(&mut pending, "room-b".to_string()));
        assert_eq!(pending.as_deref(), Some("room-a"));
    }
}
