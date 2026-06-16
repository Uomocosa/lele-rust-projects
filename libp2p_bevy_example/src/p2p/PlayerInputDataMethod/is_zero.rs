use crate::p2p::PlayerInputData;

pub fn is_zero(input: &PlayerInputData) -> bool {
    !input.left && !input.right && !input.up && !input.jump
}

#[cfg(test)]
mod tests {
    use crate::p2p::PlayerInputData;

    #[test]
    fn test_usage() {
        let input = PlayerInputData::from_bools(false, false, false, false);
        assert!(input.is_zero());
    }
}
