use std::collections::BTreeMap;

use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use super::fake_dht_fetch;
use super::fake_dht_put;

#[derive(Debug, Default, Clone, Deref, DerefMut, Serialize, Deserialize)]
pub struct FakeDht(pub BTreeMap<(String, u64), Vec<u8>>);

#[rustfmt::skip]
impl FakeDht {
    pub fn put(&mut self, lobby: String, chunk: u64, data: Vec<u8>) { fake_dht_put::put(self, lobby, chunk, data) }
    #[must_use]
    pub fn fetch(&self, lobby: &str, chunk: u64) -> Option<Vec<u8>> { fake_dht_fetch::fetch(self, lobby, chunk) }
}

#[cfg(test)]
mod tests {
    use super::FakeDht;

    #[test]
    fn test_usage() {
        let mut history = FakeDht::default();
        assert!(history.fetch("alpha", 0).is_none());
        history.put("alpha".to_string(), 0, vec![1, 2, 3]);
        assert_eq!(history.fetch("alpha", 0), Some(vec![1, 2, 3]));
        assert!(history.fetch("beta", 0).is_none());
    }
}
