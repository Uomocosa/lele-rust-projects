use bevy::prelude::Without;

use super::mesh::Mesh;
use crate::clicker;

#[must_use]
pub fn unresolved_owners(mesh: &mut Mesh, index: usize) -> Vec<u64> {
    let Some(app) = mesh.apps.get_mut(index) else {
        return Vec::new();
    };
    let mut query = app
        .world_mut()
        .query_filtered::<&clicker::Owner, Without<clicker::PlayerNo>>();
    query.iter(app.world()).map(|owner| ***owner).collect()
}

#[cfg(test)]
mod tests {
    use super::unresolved_owners;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        let before = unresolved_owners(&mut mesh, 0);
        assert_eq!(before.len(), 2, "fresh remotes are unresolved placeholders");
    }
}
