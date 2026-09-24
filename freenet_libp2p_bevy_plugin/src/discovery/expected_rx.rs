use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Debug, Deref)]
pub struct ExpectedRx(pub Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<Vec<String>>>>);

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::ExpectedRx;

    #[test]
    fn test_usage() {
        let (_exp_tx, exp_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<String>>();
        let feed = ExpectedRx(Mutex::new(Some(exp_rx)));
        assert!(feed.lock().is_ok_and(|guard| guard.is_some()));
    }
}
