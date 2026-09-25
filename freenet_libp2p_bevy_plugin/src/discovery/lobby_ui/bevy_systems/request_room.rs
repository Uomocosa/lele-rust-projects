#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::super::session::room_request::RoomRequest;
use super::super::super::session::room_request_tx::RoomRequestTx;

pub fn request_room(mut requests: MessageReader<RoomRequest>, tx: Res<RoomRequestTx>) {
    for request in requests.read() {
        if let RoomRequest::Join(room) = request {
            tx.send(room.clone()).ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::request_room;
    use crate::discovery;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<discovery::params::RoomName>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::session::RoomRequest>();
        app.insert_resource(discovery::session::RoomRequestTx(tx));
        app.add_systems(Update, request_room);
        app.world_mut()
            .resource_mut::<Messages<discovery::session::RoomRequest>>()
            .write(discovery::session::RoomRequest::Join(
                discovery::params::RoomName("room-a".to_string()),
            ));
        app.update();
        assert!(rx.try_recv().is_ok_and(|room| room.as_str() == "room-a"));
    }
}
