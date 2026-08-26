use crate::set_client;

pub fn own_count(client: &set_client::SetClient) -> u64 {
    client.seq
}

// no test_usage necessary — exercised via integration tests
