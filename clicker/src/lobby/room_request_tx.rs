use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Debug, Deref)]
pub struct RoomRequestTx(pub tokio::sync::mpsc::UnboundedSender<String>);

#[cfg(test)]
mod tests {
    use super::RoomRequestTx;

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let queue = RoomRequestTx(tx);
        assert!(queue.send("room-a".to_string()).is_ok());
        assert_eq!(rx.try_recv().unwrap_or_default(), "room-a".to_string());
    }
}
