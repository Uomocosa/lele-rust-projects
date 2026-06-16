use crate::p2p::Message;

pub fn parse_message(data: &[u8]) -> Option<Message> {
    bincode::deserialize(data).ok()
}

#[cfg(test)]
mod tests {
    use crate::p2p::Message;
    use crate::p2p::PlayerInputData;
    use crate::sync;
    use crate::sync::create_player_input_message;

    #[test]
    fn test_usage() -> Result<(), Box<dyn std::error::Error>> {
        let input = PlayerInputData::from_bools(true, false, false, false);
        let data = create_player_input_message(42, input)?;

        let parsed = sync::parse_message(&data);
        assert!(parsed.is_some(), "Should parse valid message");

        if let Some(Message::PlayerInput { tick, .. }) = parsed {
            assert_eq!(tick, 42, "Tick should match");
        }
        Ok(())
    }
}
