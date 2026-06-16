use crate::p2p::GossipTopicMethod;

#[derive(Clone, Debug)]
pub struct GossipTopic {
    pub topic: libp2p::gossipsub::IdentTopic,
}

#[rustfmt::skip]
impl GossipTopic {
    pub fn new() -> Self { GossipTopicMethod::new() }
    pub fn hash(&self) -> libp2p::gossipsub::TopicHash { GossipTopicMethod::hash(self) }
}

impl Default for GossipTopic {
    fn default() -> Self {
        Self {
            topic: libp2p::gossipsub::IdentTopic::new(crate::p2p::GAME_TOPIC_STR),
        }
    }
}

#[rustfmt::skip]
impl From<GossipTopic> for libp2p::gossipsub::IdentTopic {
    fn from(topic: GossipTopic) -> Self { GossipTopicMethod::from(topic) }
}
