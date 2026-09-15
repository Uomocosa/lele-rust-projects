use crate::clicker;

#[must_use]
pub fn click_topic(lobby: &clicker::ActiveLobby) -> String {
    format!("clicker/{}/click", **lobby)
}

#[cfg(test)]
mod tests {
    use super::click_topic;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let lobby = clicker::ActiveLobby("alpha".to_string());
        assert_eq!(click_topic(&lobby), "clicker/alpha/click");
    }
}
