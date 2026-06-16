use crate::p2p::Message;
use crate::p2p::PlayerInputData;

pub fn create_player_input_message(
    tick: u64,
    input: PlayerInputData,
) -> Result<Vec<u8>, bincode::Error> {
    let msg = Message::PlayerInput { tick, input };
    bincode::serialize(&msg)
}

#[cfg(test)]
mod tests {
    use crate::p2p::PlayerInputData;

    #[test]
    fn test_usage() -> Result<(), Box<dyn std::error::Error>> {
        let input = PlayerInputData::from_bools(true, false, false, false);
        let data = super::create_player_input_message(1, input)?;
        assert!(!data.is_empty(), "Serialized data should not be empty");
        Ok(())
    }
}
