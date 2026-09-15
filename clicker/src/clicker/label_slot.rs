use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

pub fn label_slot(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    entity: Entity,
    peer: &str,
    logical: u64,
) {
    let player = net_id::NetworkId(logical);
    let base = clicker::color_for(player);
    commands.entity(entity).insert(clicker::PlayerNo(logical));
    commands.entity(entity).insert(clicker::CursorColor(base));
    commands
        .entity(entity)
        .insert(MeshMaterial2d(materials.add(base)));
    tracing::info!(
        "cursor resolved peer={peer} player={logical} hue={:.1}",
        clicker::hue_for(player)
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::label_slot;
    use crate::clicker;

    fn label_once(
        mut commands: Commands,
        mut materials: ResMut<Assets<ColorMaterial>>,
        targets: Query<Entity, With<clicker::CursorIcon>>,
    ) {
        for entity in &targets {
            label_slot(&mut commands, &mut materials, entity, "peer", 2);
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<ColorMaterial>>();
        let target = app.world_mut().spawn(clicker::CursorIcon).id();
        app.add_systems(Update, label_once);
        app.update();
        assert_eq!(**app.world().get::<clicker::PlayerNo>(target).unwrap(), 2);
    }
}
