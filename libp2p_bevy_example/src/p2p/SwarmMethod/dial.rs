use crate::p2p::Command;
use crate::p2p::Swarm;

pub fn dial(swarm: &mut Swarm, addr: libp2p::Multiaddr) {
    swarm.command_sender.try_send(Command::Dial(addr)).ok();
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;
    use crate::p2p::Swarm;
    use libp2p::Multiaddr;

    #[test]
    fn test_usage() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        let (mut swarm, _rx) = Swarm::new(config)?;
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse()?;
        swarm.dial(addr);
        Ok(())
    }
}
