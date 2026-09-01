pub const BOX_SIZE: f32 = 40.0;
pub const MOVE_SPEED: f32 = 200.0;
pub const JUMP_SPEED: f32 = 350.0;
pub const GROUND_Y: f32 = -200.0;
pub const GROUND_TOP: f32 = GROUND_Y - GROUND_THICKNESS / 2.0;
pub const GROUND_THICKNESS: f32 = 20.0;
pub const GROUND_WIDTH: f32 = 800.0;
pub const WALL_THICKNESS: f32 = 20.0;
pub const WALL_HEIGHT: f32 = 300.0;
pub const GROUND_CHECK_DISTANCE: f32 = BOX_SIZE / 2.0 + 2.0;
pub const SPAWN_Y: f32 = GROUND_TOP + BOX_SIZE / 2.0 + 1.0;
pub const TICKS_PER_SECOND: u64 = 60;
// no test_usage necessary
