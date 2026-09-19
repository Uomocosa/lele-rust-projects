use super::mesh_heal;
use super::peer::Peer;
use super::player::Player;

pub fn heal_with(peer: &mut Peer, other: Player) {
    let Some(first) = peer.mesh.index_of(peer.player) else {
        return;
    };
    let Some(second) = peer.mesh.index_of(other) else {
        return;
    };
    mesh_heal::heal(peer.mesh, first, second);
}

#[cfg(test)]
mod tests {
    use super::super::Player;
    use super::heal_with;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        {
            let mut link = mesh.peer(1);
            link.partition_from(Player(2));
            heal_with(&mut link, Player(2));
        }
        assert!(!mesh.severed(0, 1));
    }
}
