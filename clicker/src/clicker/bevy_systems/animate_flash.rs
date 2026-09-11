use bevy::color::Mix;
use bevy::prelude::*;

use crate::clicker;

pub fn animate_flash(
    mut commands: Commands,
    time: Res<Time>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut flashing: Query<(
        Entity,
        &clicker::Owner,
        &MeshMaterial2d<ColorMaterial>,
        &mut clicker::ClickFlash,
    )>,
) {
    let materials = materials.into_inner();
    let tick = time.into_inner().delta_secs();
    for (entity, owner, handle, mut flash) in &mut flashing {
        let base = clicker::color_for(**owner);
        let next = (**flash - tick).max(0.0);
        if next <= 0.0 {
            if let Some(mut material) = materials.get_mut(handle) {
                material.color = base;
            }
            tracing::debug!("cursor flash done owner={}", ***owner);
            commands.entity(entity).remove::<clicker::ClickFlash>();
        } else {
            **flash = next;
            if let Some(mut material) = materials.get_mut(handle) {
                material.color = base.mix(&Color::WHITE, next / clicker::FLASH_SECS);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::animate_flash;
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
        app.init_resource::<Time>();
        app.init_resource::<Assets<ColorMaterial>>();
        let owner = net_id::NetworkId(1);
        let base = clicker::color_for(owner);
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::from_color(base));
        let cursor = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::Owner(owner),
                MeshMaterial2d(handle.clone()),
                clicker::ClickFlash(clicker::FLASH_SECS),
            ))
            .id();
        app.add_systems(Update, animate_flash);
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(5));
        app.update();
        let material = app
            .world()
            .resource::<Assets<ColorMaterial>>()
            .get(&handle)
            .unwrap();
        assert_ne!(
            material.color, base,
            "flash should whiten the cursor fill while active"
        );
        tracing::info!("cursor flash active, color whitened");
        let flash = app.world().get::<clicker::ClickFlash>(cursor).unwrap();
        assert!(**flash < clicker::FLASH_SECS);
    }
}
