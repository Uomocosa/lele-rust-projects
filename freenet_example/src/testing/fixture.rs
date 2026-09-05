use freenet_stdlib::prelude::*;

use crate::FreenetClient;
use crate::testing;

use testing::TestNode;

pub struct Fixture {
    pub node: TestNode,
    pub wasm: Vec<u8>,
    pub client: FreenetClient,
    pub key: ContractKey,
}

impl Fixture {
    /// # Errors
    /// Returns an error if the node or contract setup fails.
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        super::fixture_new::new().await
    }
}

// no test_usage necessary
