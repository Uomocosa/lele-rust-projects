use std::collections::BTreeMap;

use bevy::ecs::entity::Entity;
use bevy::prelude::App;

use super::engine_new;
use super::engine_player_position;
use super::engine_spawn_player;
use super::engine_step;
use super::restore;
use super::sim_state;
use crate::engine;

pub struct Engine {
    pub app: App,
    pub entities: BTreeMap<engine::PlayerId, Entity>,
    pub tick: u64,
}

impl Default for Engine {
    fn default() -> Self {
        engine_new::new()
    }
}

#[rustfmt::skip]
impl Engine {
    pub fn new() -> Self { engine_new::new() }
    pub fn step(&mut self, tick: u64, actions: &[(engine::PlayerId, engine::Action)]) -> Result<engine::Snapshot, engine::Error> { engine_step::step(self, tick, actions) }
    pub fn spawn_player(&mut self, id: engine::PlayerId) { engine_spawn_player::spawn_player(self, id) }
    pub fn player_position(&self, id: engine::PlayerId) -> Option<(f32, f32)> { engine_player_position::player_position(self, id) }
    pub fn sim_state(&mut self) -> engine::EngineSimState { sim_state::sim_state(self) }
    pub fn restore(&mut self, state: &engine::EngineSimState) { restore::restore(self, state) }
}
// no test_usage necessary — thin delegates, covered by engine_step determinism test
