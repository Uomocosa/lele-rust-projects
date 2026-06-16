use crate::p2p::GossipTopic;
use crate::p2p::GAME_TOPIC_STR;
use libp2p::gossipsub::IdentTopic;

pub fn new() -> GossipTopic {
    GossipTopic {
        topic: IdentTopic::new(GAME_TOPIC_STR),
    }
}

#[cfg(test)]
mod tests {
    use crate::p2p::GossipTopic;

    #[test]
    fn test_usage() {
        let topic = GossipTopic::new();
        assert!(!topic.hash().to_string().is_empty());
    }
}
