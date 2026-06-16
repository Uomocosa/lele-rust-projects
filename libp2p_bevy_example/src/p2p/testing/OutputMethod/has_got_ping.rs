use crate::p2p::testing::Output;

pub fn has_got_ping(output: &Output, from_tag: &str, peer_id: &str) -> bool {
    output.contains(&format!("{}:Evt:GOT_PING:{}", from_tag, peer_id))
}
