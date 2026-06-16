use bevy::prelude::*;

use crate::boxes::component::VelocityMethod;

#[derive(Component, Debug, Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[rustfmt::skip]
impl Velocity {
    pub fn new(x: f32, y: f32) -> Self { VelocityMethod::new(x, y) }
    pub fn zero() -> Self { VelocityMethod::zero() }
}
