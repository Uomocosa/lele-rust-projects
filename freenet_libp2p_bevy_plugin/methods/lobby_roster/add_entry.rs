use crate::roster;

pub fn add_entry(roster: &mut roster::LobbyRoster, lobby: String, id: [u8; 32], addr: String) {
    roster.entry(lobby).or_default().insert(id, addr);
}

#[cfg(test)]
mod tests {
    use super::add_entry;
    use crate::roster;

    #[test]
    fn test_usage() {
        let mut r = roster::LobbyRoster::default();
        add_entry(&mut r, "l".to_string(), [2u8; 32], "a".to_string());
        assert_eq!(r.len(), 1);
    }
}
