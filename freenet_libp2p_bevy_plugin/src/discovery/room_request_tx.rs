use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Debug, Deref)]
pub struct RoomRequestTx(pub tokio::sync::mpsc::UnboundedSender<String>);

#[cfg(test)]
mod tests {
    use super::RoomRequestTx;

    #[test]
    fn test_usage() {
        let (req_tx, _req_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        assert!(RoomRequestTx(req_tx).send("room-a".to_string()).is_ok());
    }
}
