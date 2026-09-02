use crate::global_counter_client;

#[must_use]
pub fn count(client: &global_counter_client::GlobalCounterClient) -> u64 {
    client.slots.values().sum()
}

// no test_usage necessary — exercised via integration tests
