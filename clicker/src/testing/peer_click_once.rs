use super::peer::Peer;

pub fn click_once(peer: &mut Peer) {
    peer.clicks(1);
}

#[cfg(test)]
mod tests {
    use super::click_once;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        {
            let mut link = mesh.peer(1);
            click_once(&mut link);
        }
        assert_eq!(testing::get_count(&mut mesh.apps[0], 1), 1);
    }
}
