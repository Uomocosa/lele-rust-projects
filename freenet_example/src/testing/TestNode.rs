pub struct TestNode {
    pub(crate) _tmp: tempfile::TempDir,
    pub(crate) port: u16,
    pub(crate) _task: tokio::task::JoinHandle<()>,
}

#[rustfmt::skip]
impl TestNode {
    pub async fn start() -> Self { crate::testing::TestNodeMethod::start().await }
    pub fn port(&self) -> u16 { crate::testing::TestNodeMethod::port(self) }
}
