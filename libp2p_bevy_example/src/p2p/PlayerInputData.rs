use serde::{Deserialize, Serialize};

use crate::p2p::PlayerInputDataMethod;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerInputData {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub jump: bool,
}

#[rustfmt::skip]
impl PlayerInputData {
    pub fn from_bools(left: bool, right: bool, up: bool, jump: bool) -> Self { PlayerInputDataMethod::from_bools(left, right, up, jump) }
    pub fn is_zero(&self) -> bool { PlayerInputDataMethod::is_zero(self) }
}
