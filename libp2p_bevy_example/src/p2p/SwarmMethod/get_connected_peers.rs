use libp2p::PeerId;
use tokio::sync::mpsc;

use crate::p2p::Command;
use crate::p2p::Swarm;

pub fn get_connected_peers(swarm: &mut Swarm) -> Vec<PeerId> {
    let (tx, mut rx) = mpsc::channel(1);
    swarm.command_sender.try_send(Command::GetPeers(tx)).ok();
    rx.blocking_recv().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;
    use crate::p2p::Swarm;

    #[test]
    fn test_usage() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        let (mut swarm, _rx) = Swarm::new(config)?;
        let peers = swarm.get_connected_peers();
        assert!(peers.is_empty());
        Ok(())
    }
}
