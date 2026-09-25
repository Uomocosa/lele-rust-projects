use bevy::prelude::Component;
use derive_more::Deref;

use super::super::params::room_name::RoomName;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
pub struct RoomButton(pub RoomName);

#[cfg(test)]
mod tests {
    use super::RoomButton;
    use crate::discovery;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app
            .world_mut()
            .spawn(RoomButton(discovery::params::RoomName(
                "room-a".to_string(),
            )))
            .id();
        app.update();
        assert!(
            app.world()
                .get::<RoomButton>(entity)
                .is_some_and(|button| button.as_str() == "room-a")
        );
    }
}
