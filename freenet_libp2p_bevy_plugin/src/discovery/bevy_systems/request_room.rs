#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::room_request::RoomRequest;
use super::super::room_request_tx::RoomRequestTx;

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
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::RoomRequest>();
        app.insert_resource(discovery::RoomRequestTx(tx));
        app.add_systems(Update, request_room);
        app.world_mut()
            .resource_mut::<Messages<discovery::RoomRequest>>()
            .write(discovery::RoomRequest::Join("room-a".to_string()));
        app.update();
        assert_eq!(rx.try_recv().unwrap_or_default(), "room-a");
    }
}
