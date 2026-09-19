use bevy::prelude::*;

use super::fake_dht;
use super::mesh_await_convergence;
use super::mesh_component;
use super::mesh_count::MeshCount;
use super::mesh_counts;
use super::mesh_heal;
use super::mesh_index_of;
use super::mesh_of;
use super::mesh_partition;
use super::mesh_route;
use super::mesh_severed;
use super::mesh_step;
use super::peer::Peer;
use super::peer_new;
use super::player::Player;

#[derive(Default)]
pub struct Mesh {
    pub apps: Vec<App>,
    pub players: Vec<Player>,
    pub blocks: Vec<(usize, usize)>,
    pub history: fake_dht::FakeDht,
}

#[rustfmt::skip]
impl Mesh {
    pub const CONVERGE_TICKS: usize = 40;

    #[must_use]
    pub fn of(peers: u32) -> Self { mesh_of::of(peers) }
    pub fn step(&mut self) { mesh_step::step(self) }
    pub fn route(&mut self) { mesh_route::route(self) }
    #[must_use]
    pub fn counts(&mut self) -> Vec<MeshCount> { mesh_counts::counts(self) }
    pub fn await_convergence(&mut self, expected: &MeshCount) -> Vec<MeshCount> { mesh_await_convergence::await_convergence(self, expected) }
    #[must_use]
    pub fn peer(&mut self, player: impl Into<Player>) -> Peer<'_> { peer_new::new(self, player.into()) }
    #[must_use]
    pub fn index_of(&self, player: impl Into<Player>) -> Option<usize> { mesh_index_of::index_of(self, player.into()) }
    pub fn partition(&mut self, first: usize, second: usize) { mesh_partition::partition(self, first, second) }
    pub fn heal(&mut self, first: usize, second: usize) { mesh_heal::heal(self, first, second) }
    #[must_use]
    pub fn severed(&self, first: usize, second: usize) -> bool { mesh_severed::severed(self, first, second) }
    #[must_use]
    pub fn component(&self, from: usize) -> Vec<usize> { mesh_component::component(self, from) }
}

#[cfg(test)]
mod tests {
    use super::Mesh;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = Mesh::of(3);
        mesh.peer(1).click_once();
        mesh.step();
        let counts = mesh.counts();
        assert_eq!(counts.len(), 3);
        assert_eq!(mesh.index_of(2), Some(1));
        let expected = testing::MeshCount::of([(testing::Player(1), 1)]);
        assert_eq!(mesh.await_convergence(&expected).len(), 3);
        mesh.route();
    }
}
