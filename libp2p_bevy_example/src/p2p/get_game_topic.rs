use crate::p2p::GossipTopic;
use libp2p::gossipsub::IdentTopic;

pub fn get_game_topic() -> IdentTopic {
    let topic = GossipTopic::new();
    topic.into()
}

#[cfg(test)]
mod tests {
    use crate::p2p;

    #[test]
    fn test_usage() {
        let topic = p2p::get_game_topic();
        let hash = topic.hash();
        let hash_str = hash.to_string();
        assert!(!hash_str.is_empty(), "Topic hash should not be empty");
    }
}
