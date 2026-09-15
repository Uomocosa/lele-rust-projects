use bevy::prelude::*;

use super::fake_dht;
use super::mesh_click;
use super::mesh_component;
use super::mesh_count::MeshCount;
use super::mesh_counts;
use super::mesh_heal;
use super::mesh_partition;
use super::mesh_route;
use super::mesh_severed;
use super::mesh_step;
use super::mesh_three;

#[derive(Default)]
pub struct Mesh {
    pub apps: [App; 3],
    pub names: [String; 3],
    pub blocks: Vec<(usize, usize)>,
    pub history: fake_dht::FakeDht,
}

#[rustfmt::skip]
impl Mesh {
    #[must_use]
    pub fn three() -> Self { mesh_three::three() }
    pub fn step(&mut self) { mesh_step::step(self) }
    pub fn route(&mut self) { mesh_route::route(self) }
    #[must_use]
    pub fn counts(&mut self) -> Vec<MeshCount> { mesh_counts::counts(self) }
    pub fn click(&mut self, owner: u64, times: u32) { mesh_click::click(self, owner, times) }
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

    #[test]
    fn test_usage() {
        let mut mesh = Mesh::three();
        mesh.click(1, 1);
        mesh.step();
        let counts = mesh.counts();
        assert_eq!(counts.len(), 3);
        mesh.route();
    }
}
