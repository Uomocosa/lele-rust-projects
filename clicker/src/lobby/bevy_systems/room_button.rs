use bevy::prelude::Component;
use derive_more::Deref;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
pub struct RoomButton(pub String);

#[cfg(test)]
mod tests {
    use super::RoomButton;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(RoomButton("room-a".to_string())).id();
        app.update();
        let button = app.world().get::<RoomButton>(entity).expect("button");
        assert_eq!(**button, "room-a");
    }
}
