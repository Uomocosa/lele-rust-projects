use bevy::prelude::Resource;

use crate::sync::resource::TickMethod;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Tick {
    pub current: u64,
}

#[rustfmt::skip]
impl Tick {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 { TickMethod::next(self) }
}
