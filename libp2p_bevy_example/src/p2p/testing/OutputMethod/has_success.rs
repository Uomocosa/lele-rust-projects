use crate::p2p::testing::Output;

pub fn has_success(output: &Output, tag: &str) -> bool {
    output.contains(&format!("{}:Evt:SUCCESS", tag))
}
