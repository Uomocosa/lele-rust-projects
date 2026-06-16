use crate::p2p::testing::OutputMethod;

pub struct Output {
    pub lines: Vec<String>,
}

#[rustfmt::skip]
impl Output {
    pub fn contains(&self, line: &str) -> bool { OutputMethod::contains(self, line) }
    pub fn peer_id(&self, tag: &str) -> Option<String> { OutputMethod::peer_id(self, tag) }
    pub fn has_discovered(&self, from_tag: &str, peer_id: &str) -> bool { OutputMethod::has_discovered(self, from_tag, peer_id) }
    pub fn has_connected(&self, from_tag: &str, peer_id: &str) -> bool { OutputMethod::has_connected(self, from_tag, peer_id) }
    pub fn has_got_ping(&self, from_tag: &str, peer_id: &str) -> bool { OutputMethod::has_got_ping(self, from_tag, peer_id) }
    pub fn has_success(&self, tag: &str) -> bool { OutputMethod::has_success(self, tag) }
    pub fn has_sent_ping(&self, tag: &str) -> bool { OutputMethod::has_sent_ping(self, tag) }
}
