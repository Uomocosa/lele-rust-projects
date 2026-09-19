use super::mesh_partition;
use super::peer::Peer;
use super::player::Player;

pub fn partition_from(peer: &mut Peer, other: Player) {
    let Some(first) = peer.mesh.index_of(peer.player) else {
        return;
    };
    let Some(second) = peer.mesh.index_of(other) else {
        return;
    };
    mesh_partition::partition(peer.mesh, first, second);
}

#[cfg(test)]
mod tests {
    use super::Player;
    use super::partition_from;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        {
            let mut link = mesh.peer(1);
            partition_from(&mut link, Player(2));
        }
        assert!(mesh.severed(0, 1));
    }
}
