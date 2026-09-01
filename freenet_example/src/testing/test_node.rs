pub struct TestNode {
    pub(crate) _tmp: tempfile::TempDir,
    pub port: u16,
    pub(crate) _task: tokio::task::JoinHandle<()>,
}

#[rustfmt::skip]
impl TestNode {
    /// # Errors
    /// Returns an error if the embedded node fails to start.
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> { super::test_node_method::start().await }
}

// no test_usage necessary — exercised via integration tests
