use std::path::Path;

use tokio::task::JoinHandle;

use crate::methods::test_node as tn_method;

pub struct TestNode {
    pub _tmp: Option<tempfile::TempDir>,
    pub(crate) port: u16,
    pub _task: JoinHandle<()>,
}

#[rustfmt::skip]
impl TestNode {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        tn_method::start().await
    }
    pub async fn start_at(dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        tn_method::start_at(dir).await
    }
    pub async fn shutdown(self) { tn_method::shutdown(self).await }
    pub fn port(&self) -> u16 { tn_method::port(self) }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        self._task.abort();
    }
}
