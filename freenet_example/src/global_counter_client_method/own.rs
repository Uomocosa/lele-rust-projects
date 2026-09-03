use crate::global_counter_client;

#[must_use]
pub fn own(client: &global_counter_client::GlobalCounterClient) -> u64 {
    client.slots.get(&client.pubkey).copied().unwrap_or(0)
}

// no test_usage necessary — exercised via integration tests
