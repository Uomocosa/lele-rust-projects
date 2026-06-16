use crate::boxes::component::PlayerInput;
use crate::p2p::PlayerInputData;

pub fn new() -> PlayerInput {
    PlayerInput {
        input: PlayerInputData::default(),
    }
}

#[cfg(test)]
mod tests {
    use crate::boxes::component::PlayerInput;

    #[test]
    fn test_usage() {
        let player_input = PlayerInput::new();
        assert!(player_input.input.is_zero());
    }
}
