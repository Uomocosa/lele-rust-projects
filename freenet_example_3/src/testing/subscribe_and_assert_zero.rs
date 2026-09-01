use freenet_stdlib::prelude::*;

use crate::FreenetClient;
use crate::testing;

/// # Errors
/// Returns an error if subscription fails or the initial count is not zero.
pub async fn subscribe_and_assert_zero(
    client: &mut FreenetClient,
    key: ContractKey,
) -> Result<(), String> {
    let count = testing::subscribe(client, key)
        .await
        .map_err(|e| e.to_string())?;
    if count == 0 {
        Ok(())
    } else {
        Err(format!("expected initial count 0, got {count}"))
    }
}

#[cfg(test)]
mod tests {
    use super::subscribe_and_assert_zero;
    use crate::testing::TestNode;
    use crate::testing::connect;
    use crate::testing::deploy;
    use crate::testing::load_wasm;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let wasm = load_wasm();
        let mut publisher = connect(node.port).await.unwrap();
        let key = deploy(&mut publisher, &wasm).await.unwrap();
        let mut sub = connect(node.port).await.unwrap();
        subscribe_and_assert_zero(&mut sub, key).await.unwrap();
    }
}
