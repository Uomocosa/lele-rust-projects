use super::roster::Roster;

pub fn remove_entry(roster: &mut Roster, lobby: &str, id: [u8; 32]) -> bool {
    roster
        .get_mut(lobby)
        .is_some_and(|members| members.remove(&id).is_some())
}

#[cfg(test)]
mod tests {
    use super::remove_entry;
    use crate::roster::Roster;

    #[test]
    fn test_usage() {
        let mut roster = Roster::default();
        roster.add_entry("lobby".to_string(), [3u8; 32], "addr".to_string());
        assert!(remove_entry(&mut roster, "lobby", [3u8; 32]));
        assert!(!remove_entry(&mut roster, "lobby", [3u8; 32]));
        assert!(!remove_entry(&mut roster, "missing", [3u8; 32]));
    }
}
