use crate::clicker;

#[must_use]
pub fn gossip_roster_topic(lobby: &clicker::ActiveLobby) -> String {
    format!("clicker/{}/roster", **lobby)
}

#[cfg(test)]
mod tests {
    use super::gossip_roster_topic;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let lobby = clicker::ActiveLobby("alpha".to_string());
        assert_eq!(gossip_roster_topic(&lobby), "clicker/alpha/roster");
    }
}
