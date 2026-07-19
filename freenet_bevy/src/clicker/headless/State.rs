use bevy::prelude::Resource;

#[derive(Resource)]
pub struct HeadlessState {
    pub max_ticks: usize,
    pub pending: bool,
    pub completed: usize,
}

#[derive(Clone)]
pub struct HeadlessConfig {
    pub max_ticks: usize,
}
