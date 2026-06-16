use crate::p2p::testing::Output;

pub fn has_discovered(output: &Output, from_tag: &str, peer_id: &str) -> bool {
    output.contains(&format!("{}:Evt:DISCOVERED:{}", from_tag, peer_id))
}
