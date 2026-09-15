use super::fake_dht::FakeDht;

#[must_use]
pub fn fetch(history: &FakeDht, lobby: &str, chunk: u64) -> Option<Vec<u8>> {
    history.get(&(lobby.to_string(), chunk)).cloned()
}
// no test_usage necessary
