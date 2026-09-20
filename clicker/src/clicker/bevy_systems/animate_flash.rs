use bevy::color::Mix;
use bevy::prelude::*;

use crate::clicker;

pub fn animate_flash(
    mut commands: Commands,
    time: Res<Time>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut flashing: Query<(
        Entity,
        &clicker::CursorColor,
        &MeshMaterial2d<ColorMaterial>,
        &mut clicker::ClickFlash,
    )>,
    numbers: Query<&clicker::PlayerNo>,
) {
    let materials = materials.into_inner();
    let tick = time.into_inner().delta_secs();
    for (entity, base_color, handle, mut flash) in &mut flashing {
        let base = **base_color;
        let next = (**flash - tick).max(0.0);
        let player = numbers
            .get(entity)
            .ok()
            .map_or_else(|| "?".to_string(), |n| format!("{}", **n));
        if next <= 0.0 {
            if let Some(mut material) = materials.get_mut(handle) {
                material.color = base;
            }
            tracing::info!("cursor flash done player={player} color={base:?}");
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
            filter: "info,clicker=debug,clicker_lib=debug".to_string(),
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
                clicker::CursorColor(base),
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

    #[test]
    fn flash_restores_resolved_base_not_owner() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.init_resource::<Assets<ColorMaterial>>();
        let sender = net_id::NetworkId::from_peer("peer");
        let base = clicker::color_for(net_id::NetworkId(5));
        assert_ne!(base, clicker::color_for(sender));
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::from_color(Color::WHITE));
        app.world_mut().spawn((
            clicker::CursorIcon,
            clicker::Owner(sender),
            clicker::PlayerNo(5),
            clicker::CursorColor(base),
            MeshMaterial2d(handle.clone()),
            clicker::ClickFlash(0.0),
        ));
        app.add_systems(Update, animate_flash);
        app.update();
        let material = app
            .world()
            .resource::<Assets<ColorMaterial>>()
            .get(&handle)
            .unwrap();
        assert_eq!(material.color, base);
    }
}
