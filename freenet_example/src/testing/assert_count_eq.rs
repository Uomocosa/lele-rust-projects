use freenet_stdlib::prelude::*;

use crate::FreenetClient;
use crate::testing;

/// # Errors
/// Returns an error if `get_count` fails or the count does not match.
pub async fn assert_count_eq(
    client: &mut FreenetClient,
    key: ContractKey,
    expected: u64,
) -> Result<(), String> {
    let count = testing::get_count(client, key)
        .await
        .map_err(|e| e.to_string())?;
    if count == expected {
        Ok(())
    } else {
        Err(format!("expected count {expected}, got {count}"))
    }
}

#[cfg(test)]
mod tests {
    use super::assert_count_eq;
    use crate::testing::TestNode;
    use crate::testing::connect;
    use crate::testing::deploy;
    use crate::testing::load_wasm;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let wasm = load_wasm();
        let mut client = connect(node.port).await.unwrap();
        let key = deploy(&mut client, &wasm).await.unwrap();
        assert_count_eq(&mut client, key, 0).await.unwrap();
    }
}
