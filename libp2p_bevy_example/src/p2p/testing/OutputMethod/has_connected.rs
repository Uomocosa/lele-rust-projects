use crate::p2p::testing::Output;

pub fn has_connected(output: &Output, from_tag: &str, peer_id: &str) -> bool {
    output.contains(&format!("{}:Evt:CONNECTED:{}", from_tag, peer_id))
}
