pub struct TestNode {
    pub(crate) _tmp: tempfile::TempDir,
    pub(crate) port: u16,
    pub(crate) _task: tokio::task::JoinHandle<()>,
}

#[rustfmt::skip]
impl TestNode {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> { super::test_node_method::start().await }
    pub fn port(&self) -> u16 { super::test_node_method::port(self) }
}

// no test_usage necessary — exercised via integration tests
