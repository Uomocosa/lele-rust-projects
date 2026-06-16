use bevy::prelude::*;

use crate::boxes::component::PositionMethod;

#[derive(Component, Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[rustfmt::skip]
impl Position {
    pub fn new(x: f32, y: f32) -> Self { PositionMethod::new(x, y) }
    pub fn zero() -> Self { PositionMethod::zero() }
}
