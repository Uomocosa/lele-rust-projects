use super::click_times;
use super::peer::Peer;
use crate::clicker;

pub fn clicks(peer: &mut Peer, times: u32) {
    let owner = *peer.player;
    for app in &mut peer.mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == owner {
            click_times(app, times);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::clicks;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        {
            let mut link = mesh.peer(2);
            clicks(&mut link, 4);
        }
        let counts = mesh.counts();
        assert!(counts.iter().any(|c| c.count(2) == 4));
        assert!(counts.iter().all(|c| c.count(1) == 0));
    }
}
