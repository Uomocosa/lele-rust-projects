use serde::{Deserialize, Serialize};

use crate::engine;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub direction: engine::Direction,
    pub jump: bool,
}

#[rustfmt::skip]
impl Action {
    pub fn move_value(self) -> f32 { engine::action_move_value::move_value(self) }
    pub fn is_null(self) -> bool { engine::action_is_null::is_null(self) }
}
// no test_usage necessary — trivial delegates, covered in action_move_value/action_is_null
