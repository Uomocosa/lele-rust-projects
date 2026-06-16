use crate::p2p::GossipTopic;
use libp2p::gossipsub::IdentTopic;

pub fn from(topic: GossipTopic) -> IdentTopic {
    topic.topic
}

#[cfg(test)]
mod tests {
    use crate::p2p::GossipTopic;
    use crate::p2p::GAME_TOPIC_STR;
    use libp2p::gossipsub::IdentTopic;

    #[test]
    fn test_usage() {
        let topic = GossipTopic::new();
        let ident: IdentTopic = topic.into();
        assert_eq!(ident.to_string(), GAME_TOPIC_STR);
    }
}
