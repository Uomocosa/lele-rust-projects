use tokio::task::JoinHandle;

use crate::methods::testing::test_node as tn_method;

pub struct TestNode {
    pub _tmp: tempfile::TempDir,
    pub(crate) port: u16,
    pub _task: JoinHandle<()>,
}

#[rustfmt::skip]
impl TestNode {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        tn_method::start().await
    }
    pub fn port(&self) -> u16 { tn_method::port(self) }
}
