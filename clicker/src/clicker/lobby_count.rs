use freenet_libp2p_bevy_plugin::roster;

#[must_use]
pub fn lobby_count(roster: &roster::Roster, lobby: &str) -> usize {
    roster.get(lobby).map_or(0, std::collections::BTreeMap::len)
}

#[cfg(test)]
mod tests {
    use super::lobby_count;
    use freenet_libp2p_bevy_plugin::roster;

    #[test]
    fn test_usage() {
        let roster = roster::Roster::default();
        assert_eq!(lobby_count(&roster, "alpha"), 0);
        let mut roster = roster;
        roster.add_entry("alpha".to_string(), [1u8; 32], "peer".to_string());
        assert_eq!(lobby_count(&roster, "alpha"), 1);
        assert_eq!(lobby_count(&roster, "beta"), 0);
    }
}
