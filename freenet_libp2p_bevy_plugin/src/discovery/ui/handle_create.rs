use bevy::prelude::*;

use crate::discovery;
use discovery::ui::CreateRoomButton;

pub fn handle_create(
    buttons: Query<&Interaction, (Changed<Interaction>, With<CreateRoomButton>)>,
    mut commands: MessageWriter<discovery::Command>,
    mut counter: Local<u32>,
) {
    for interaction in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        *counter = counter.saturating_add(1);
        let name = format!("room-{}-{}", *discovery::now_epoch(), *counter);
        tracing::info!(target: "room_lobby", "ui create room={name}");
        commands.write(discovery::Command::Create(discovery::RoomName(name)));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::handle_create;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<discovery::Command>();
        app.world_mut()
            .spawn((discovery::ui::CreateRoomButton, Interaction::Pressed));
        app.add_systems(Update, handle_create);
        app.update();
        let messages = app.world().resource::<Messages<discovery::Command>>();
        let mut cursor = messages.get_cursor();
        let sent: Vec<_> = cursor.read(messages).collect();
        assert!(matches!(
            sent.as_slice(),
            [discovery::Command::Create(name)] if name.starts_with("room-")
        ));
    }
}
