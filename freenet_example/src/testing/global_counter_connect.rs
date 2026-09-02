use crate::Role;
use crate::global_counter_client;
use crate::global_counter_error;

use global_counter_client::GlobalCounterClient;

/// # Errors
/// Returns an error if the `GlobalCounterClient` connection fails.
pub async fn global_counter_connect(
    port: u16,
    wasm: &[u8],
    role: Role,
) -> Result<GlobalCounterClient, global_counter_error::GlobalCounterError> {
    GlobalCounterClient::connect("127.0.0.1", port, wasm, role).await
}

#[cfg(test)]
mod tests {
    use super::global_counter_connect;
    use crate::Role;
    use crate::testing::TestNode;
    use crate::testing::load_wasm;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let wasm = load_wasm();
        let client = global_counter_connect(node.port, &wasm, Role::Publish)
            .await
            .unwrap();
        assert_eq!(client.count(), 0);
    }
}
