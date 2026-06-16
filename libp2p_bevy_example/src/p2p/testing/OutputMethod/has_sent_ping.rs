use crate::p2p::testing::Output;

pub fn has_sent_ping(output: &Output, tag: &str) -> bool {
    output.contains(&format!("{}:Evt:SENT_PING", tag))
}
