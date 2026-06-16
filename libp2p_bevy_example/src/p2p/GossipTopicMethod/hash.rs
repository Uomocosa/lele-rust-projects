use crate::p2p::GossipTopic;

pub fn hash(topic: &GossipTopic) -> libp2p::gossipsub::TopicHash {
    topic.topic.hash()
}

#[cfg(test)]
mod tests {
    use crate::p2p::GossipTopic;

    #[test]
    fn test_usage() {
        let topic = GossipTopic::new();
        let hash = topic.hash();
        assert!(!hash.to_string().is_empty());
    }
}
