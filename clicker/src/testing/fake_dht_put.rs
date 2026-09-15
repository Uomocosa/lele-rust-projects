use super::fake_dht::FakeDht;

pub fn put(history: &mut FakeDht, lobby: String, chunk: u64, data: Vec<u8>) {
    history.insert((lobby, chunk), data);
}
// no test_usage necessary
