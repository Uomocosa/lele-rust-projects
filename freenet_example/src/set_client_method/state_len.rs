use crate::set_client;

#[must_use]
pub fn state_len(client: &set_client::SetClient) -> u64 {
    u64::try_from(client.set.len()).unwrap_or(u64::MAX)
}

// no test_usage necessary — exercised via integration tests
