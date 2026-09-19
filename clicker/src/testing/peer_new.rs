use super::mesh::Mesh;
use super::peer::Peer;
use super::player::Player;

#[must_use]
pub const fn new(mesh: &mut Mesh, player: Player) -> Peer<'_> {
    Peer { mesh, player }
}

#[cfg(test)]
mod tests {
    use super::Player;
    use super::new;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        {
            let mut link = new(&mut mesh, Player(1));
            link.click_once();
        }
        assert_eq!(testing::get_count(&mut mesh.apps[0], 1), 1);
    }
}
