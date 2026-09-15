use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;

use crate::discovery;

#[derive(Resource, Debug, Deref)]
pub struct DirectoryFeed(
    pub Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<discovery::DirectoryState>>>,
);

#[cfg(test)]
mod tests {
    use super::DirectoryFeed;
    use crate::discovery;
    use std::sync::Mutex;

    #[test]
    fn test_usage() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<discovery::DirectoryState>();
        let feed = DirectoryFeed(Mutex::new(Some(rx)));
        assert!(feed.lock().is_ok_and(|guard| guard.is_some()));
        drop(tx);
        assert!(feed.lock().is_ok_and(|guard| guard.is_some()));
    }
}
