use crate::clicker_client;
pub fn count(client: &clicker_client::ClickerClient) -> u64 {
    client.count
}

// no test_usage necessary — exercised via integration tests
