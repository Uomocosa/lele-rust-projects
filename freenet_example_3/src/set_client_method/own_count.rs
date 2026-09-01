use crate::set_client;

#[must_use]
pub const fn own_count(client: &set_client::SetClient) -> u64 {
    client.seq
}

// no test_usage necessary — exercised via integration tests
