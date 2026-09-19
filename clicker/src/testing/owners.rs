use super::mesh::Mesh;
use crate::clicker;

#[must_use]
pub fn owners(mesh: &mut Mesh, index: usize) -> Vec<u64> {
    let Some(app) = mesh.apps.get_mut(index) else {
        return Vec::new();
    };
    let mut query = app.world_mut().query::<&clicker::Owner>();
    query.iter(app.world()).map(|owner| ***owner).collect()
}

#[cfg(test)]
mod tests {
    use super::owners;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        assert!(owners(&mut mesh, 0).contains(&1));
    }
}
