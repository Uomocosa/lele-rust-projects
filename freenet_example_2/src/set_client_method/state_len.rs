use crate::set_client;

pub fn state_len(client: &set_client::SetClient) -> u64 {
    client.set.len() as u64
}

// no test_usage necessary — exercised via integration tests
