use crate::clicker;

#[must_use]
pub fn pos_topic(lobby: &clicker::ActiveLobby) -> String {
    format!("clicker/{}/pos", **lobby)
}

#[cfg(test)]
mod tests {
    use super::pos_topic;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let lobby = clicker::ActiveLobby("alpha".to_string());
        assert_eq!(pos_topic(&lobby), "clicker/alpha/pos");
    }
}
