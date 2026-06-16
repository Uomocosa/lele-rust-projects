use crate::p2p::PlayerInputData;

pub fn from_bools(left: bool, right: bool, up: bool, jump: bool) -> PlayerInputData {
    PlayerInputData {
        left,
        right,
        up,
        jump,
    }
}

#[cfg(test)]
mod tests {
    use crate::p2p::PlayerInputData;

    #[test]
    fn test_usage() {
        let input = PlayerInputData::from_bools(true, false, true, false);
        assert!(input.left);
        assert!(!input.right);
        assert!(input.up);
        assert!(!input.jump);
    }
}
