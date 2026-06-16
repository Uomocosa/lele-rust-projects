use bevy::prelude::*;

use crate::clicker::component::ClickCounterMethod;

#[derive(Component)]
pub struct ClickCounter {
    pub count: u32,
}

#[rustfmt::skip]
impl ClickCounter {
    pub fn increment(&mut self) { ClickCounterMethod::increment(self) }
    pub fn decrement(&mut self) { ClickCounterMethod::decrement(self) }
}
