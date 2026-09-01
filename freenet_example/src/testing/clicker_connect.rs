use crate::Role;
use crate::clicker_client;
use crate::clicker_error;

use clicker_client::ClickerClient;

/// # Errors
/// Returns an error if the `ClickerClient` connection fails.
pub async fn clicker_connect(
    port: u16,
    wasm: &[u8],
    role: Role,
) -> Result<ClickerClient, clicker_error::ClickerError> {
    ClickerClient::connect("127.0.0.1", port, wasm, role).await
}

#[cfg(test)]
mod tests {
    use super::clicker_connect;
    use crate::Role;
    use crate::testing::TestNode;
    use crate::testing::load_wasm;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let wasm = load_wasm();
        let client = clicker_connect(node.port, &wasm, Role::Publish)
            .await
            .unwrap();
        assert_eq!(client.count(), 0);
    }
}
