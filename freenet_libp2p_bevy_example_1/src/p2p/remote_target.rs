use bevy::math::Vec2;
use bevy::prelude::Component;

#[derive(Component, Debug, Clone, Copy)]
pub struct RemoteTarget {
    pub pos: Vec2,
    pub tick: u64,
    pub sent_at_ms: u64,
}
