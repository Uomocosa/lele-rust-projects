use crate::p2p::Command;
use crate::p2p::Message;
use crate::p2p::Swarm;
use libp2p::gossipsub::IdentTopic;

pub fn publish(swarm: &mut Swarm, topic: IdentTopic, message: Message) {
    swarm
        .command_sender
        .try_send(Command::Publish(topic, message))
        .ok();
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;
    use crate::p2p::Message;
    use crate::p2p::Swarm;
    use libp2p::gossipsub::IdentTopic;
    use libp2p::PeerId;

    #[test]
    fn test_usage() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        let (mut swarm, _rx) = Swarm::new(config)?;
        let topic = IdentTopic::new("test");
        swarm.publish(
            topic,
            Message::Ping {
                peer_id: PeerId::random(),
            },
        );
        Ok(())
    }
}
