use bevy::prelude::*;

use crate::clicker;

pub fn emit_flash(
    mut commands: Commands,
    changed: Query<(&clicker::Owner, &clicker::ClickCounter), Changed<clicker::ClickCounter>>,
    cursors: Query<(Entity, &clicker::Owner), With<clicker::CursorIcon>>,
) {
    for (owner, _) in &changed {
        for (cursor, cursor_owner) in &cursors {
            if **cursor_owner == **owner {
                tracing::debug!("cursor flash emit owner={}", ***owner);
                commands
                    .entity(cursor)
                    .insert(clicker::ClickFlash(clicker::FLASH_SECS));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::emit_flash;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    // needed helper: enables debug logs for this test only
    fn test_logging(app: &mut App) {
        app.add_plugins(bevy::log::LogPlugin {
            level: bevy::log::Level::DEBUG,
            filter: "info,clicker_lib=debug".to_string(),
            ..default()
        });
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        test_logging(&mut app);
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter(3),
        ));
        let cursor = app
            .world_mut()
            .spawn((clicker::CursorIcon, clicker::Owner(net_id::NetworkId(1))))
            .id();
        app.add_systems(Update, emit_flash);
        app.update();
        let flash = app.world().get::<clicker::ClickFlash>(cursor).unwrap();
        assert_eq!(**flash, clicker::FLASH_SECS);
    }
}
