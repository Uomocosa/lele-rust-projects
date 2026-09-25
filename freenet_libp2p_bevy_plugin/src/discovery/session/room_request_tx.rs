use bevy::prelude::Resource;
use derive_more::Deref;

use super::super::params::room_name::RoomName;

#[derive(Resource, Debug, Deref)]
pub struct RoomRequestTx(pub tokio::sync::mpsc::UnboundedSender<RoomName>);

#[cfg(test)]
mod tests {
    use super::RoomRequestTx;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let (req_tx, _req_rx) =
            tokio::sync::mpsc::unbounded_channel::<discovery::params::RoomName>();
        assert!(
            RoomRequestTx(req_tx)
                .send(discovery::params::RoomName("room-a".to_string()))
                .is_ok()
        );
    }
}
