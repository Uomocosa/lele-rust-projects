use super::super::params::room_name::RoomName;

#[must_use]
pub fn should_switch(current: &RoomName, requested: &RoomName) -> bool {
    !requested.is_empty() && current != requested
}

#[cfg(test)]
mod tests {
    use super::should_switch;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let current = discovery::params::RoomName("room-a".to_string());
        let other = discovery::params::RoomName("room-b".to_string());
        assert!(should_switch(&current, &other));
        assert!(!should_switch(&current, &current));
        assert!(!should_switch(
            &current,
            &discovery::params::RoomName(String::new())
        ));
    }
}
