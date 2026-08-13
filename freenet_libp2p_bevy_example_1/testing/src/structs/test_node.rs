use tokio::task::JoinHandle;

use crate::methods::test_node as tn_method;

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
        tn_method::start_gateway(public_port).await
    }
    pub async fn start_peer(gateway_public_port: u16, gateway_public_key_hex: &str) -> Result<Self, Box<dyn std::error::Error>> {
        tn_method::start_peer(gateway_public_port, gateway_public_key_hex).await
    }
    pub async fn shutdown(self) { tn_method::shutdown(self).await }
    pub fn port(&self) -> u16 { tn_method::port(self) }
    pub fn public_port(&self) -> u16 { tn_method::public_port(self) }
    pub fn public_key_hex(&self) -> &str { tn_method::public_key_hex(self) }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        self._task.abort();
    }
}
