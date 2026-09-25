#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ready {
    pub peer_id: String,
    pub addrs: Vec<String>,
}
