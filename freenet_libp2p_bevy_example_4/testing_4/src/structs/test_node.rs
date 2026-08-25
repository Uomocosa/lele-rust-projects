use tokio::task::JoinHandle;

use crate::methods;

pub struct TestNode {
    pub _tmp: Option<tempfile::TempDir>,
    pub(crate) port: u16,
    pub(crate) public_port: u16,
    pub(crate) public_key_hex: String,
    pub _task: JoinHandle<()>,
    pub(crate) shutdown_handle: freenet::ShutdownHandle,
}

#[rustfmt::skip]
impl TestNode {
    pub async fn start_gateway(public_port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        methods::test_node::start_gateway(public_port).await
    }
    pub async fn start_peer(gateway_public_port: u16, gateway_public_key_hex: &str) -> Result<Self, Box<dyn std::error::Error>> {
        methods::test_node::start_peer(gateway_public_port, gateway_public_key_hex).await
    }
    pub async fn shutdown(self) { methods::test_node::shutdown(self).await }
    pub fn port(&self) -> u16 { methods::test_node::port(self) }
    pub fn public_port(&self) -> u16 { methods::test_node::public_port(self) }
    pub fn public_key_hex(&self) -> &str { methods::test_node::public_key_hex(self) }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        self._task.abort();
    }
}
