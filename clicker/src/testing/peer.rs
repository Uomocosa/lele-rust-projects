use super::mesh::Mesh;
use super::peer_click_once;
use super::peer_clicks;
use super::peer_heal_with;
use super::peer_partition_from;
use super::player::Player;

pub struct Peer<'a> {
    pub mesh: &'a mut Mesh,
    pub player: Player,
}

#[rustfmt::skip]
impl Peer<'_> {
    pub fn clicks(&mut self, times: u32) { peer_clicks::clicks(self, times) }
    pub fn click_once(&mut self) { peer_click_once::click_once(self) }
    pub fn partition_from(&mut self, other: impl Into<Player>) { peer_partition_from::partition_from(self, other.into()) }
    pub fn heal_with(&mut self, other: impl Into<Player>) { peer_heal_with::heal_with(self, other.into()) }
}

#[cfg(test)]
mod tests {
    use super::Peer;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        {
            let mut link = Peer {
                mesh: &mut mesh,
                player: testing::Player(1),
            };
            link.click_once();
        }
        assert_eq!(testing::get_count(&mut mesh.apps[0], 1), 1);
    }
}
