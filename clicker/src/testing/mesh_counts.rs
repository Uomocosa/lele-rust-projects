use bevy::prelude::*;

use super::mesh::Mesh;
use super::mesh_count::MeshCount;
use super::player::Player;
use crate::clicker;

pub fn counts(mesh: &mut Mesh) -> Vec<MeshCount> {
    mesh.apps.iter_mut().map(count_one).collect()
}

// needed helper: reads logical counters plus global of one app
fn count_one(app: &mut App) -> MeshCount {
    let mut out = MeshCount::default();
    let mut query = app
        .world_mut()
        .query::<(&clicker::PlayerNo, &clicker::ClickCounter)>();
    for (player, counter) in query.iter(app.world()) {
        if **counter != 0 {
            out.per.insert(Player(**player), **counter);
        }
    }
    out.global = **app.world().resource::<clicker::GlobalCounter>();
    out
}

#[cfg(test)]
mod tests {
    use super::counts;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        let counts = counts(&mut mesh);
        assert_eq!(counts.len(), 3);
        assert!(counts.iter().all(|c| *c == testing::MeshCount::default()));
    }
}
