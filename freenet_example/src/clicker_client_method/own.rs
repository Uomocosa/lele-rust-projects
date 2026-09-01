use crate::clicker_client;

#[must_use]
pub fn own(client: &clicker_client::ClickerClient) -> u64 {
    client.slots.get(&client.tag).copied().unwrap_or(0)
}

// no test_usage necessary — exercised via integration tests
