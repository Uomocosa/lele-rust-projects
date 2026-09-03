use crate::testing;

use testing::TestNode;

pub async fn new() -> Result<super::fixture::Fixture, Box<dyn std::error::Error>> {
    let node = TestNode::start().await?;
    let wasm = testing::load_wasm();
    let mut client = testing::connect(node.port).await?;
    let key = testing::deploy(&mut client, &wasm).await?;
    Ok(super::fixture::Fixture {
        node,
        wasm,
        client,
        key,
    })
}

#[cfg(test)]
mod tests {
    use super::new;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let fixture = new().await.unwrap();
        assert_ne!(fixture.key.id().as_bytes().len(), 0);
    }
}
