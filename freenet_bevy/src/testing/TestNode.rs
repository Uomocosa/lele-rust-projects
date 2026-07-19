use tokio::task::JoinHandle;

pub struct TestNode {
    pub _tmp: tempfile::TempDir,
    pub(crate) port: u16,
    pub _task: JoinHandle<()>,
}

#[rustfmt::skip]
impl TestNode {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        crate::testing::TestNodeMethod::start().await
    }
    pub fn port(&self) -> u16 { crate::testing::TestNodeMethod::port(self) }
}
