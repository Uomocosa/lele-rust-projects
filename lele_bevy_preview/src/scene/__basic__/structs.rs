use bevy::ecs::world::World;
use bevy::prelude::App;

pub struct State {
    pub label: String,
    pub apply: fn(&mut World),
}

pub struct Timeline {
    pub label: String,
    pub frames: u32,
    pub fps: u32,
    pub apply: fn(&mut World),
}

pub struct Scene {
    pub name: String,
    pub build: fn(&mut App),
    pub states: Vec<State>,
    pub timeline: Option<Timeline>,
}
