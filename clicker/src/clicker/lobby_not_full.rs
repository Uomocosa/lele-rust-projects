use freenet_libp2p_bevy_plugin::roster;

#[must_use]
pub fn lobby_not_full(roster: &roster::Roster, cap: usize) -> Vec<String> {
    let mut open = Vec::new();
    for (lobby, members) in roster.iter() {
        if members.len() < cap {
            open.push(lobby.clone());
        }
    }
    open.sort();
    open
}

#[cfg(test)]
mod tests {
    use super::lobby_not_full;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::roster;
    #[test]
    fn test_usage() {
        let mut roster = roster::Roster::default();
        roster.add_entry("alpha".to_string(), [1u8; 32], "a".to_string());
        roster.add_entry("beta".to_string(), [2u8; 32], "b".to_string());
        roster.add_entry("beta".to_string(), [3u8; 32], "c".to_string());
        assert_eq!(lobby_not_full(&roster, 2), vec!["alpha".to_string()]);
        assert_eq!(clicker::lobby_count(&roster, "beta"), 2);
    }
}
