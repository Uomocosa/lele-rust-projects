use super::mesh::Mesh;
use super::player::Player;

#[must_use]
pub fn index_of(mesh: &Mesh, player: Player) -> Option<usize> {
    mesh.players
        .iter()
        .position(|candidate| *candidate == player)
}

#[cfg(test)]
mod tests {
    use super::Player;
    use super::index_of;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mesh = testing::Mesh::of(3);
        assert_eq!(index_of(&mesh, Player(1)), Some(0));
        assert_eq!(index_of(&mesh, Player(9)), None);
    }
}
