use bevy::color::Mix;
use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

pub fn animate_flash(
    mut commands: Commands,
    time: Res<Time>,
    own: Res<net_id::NetworkId>,
    mut flashing: Query<(
        Entity,
        &clicker::Owner,
        &mut Sprite,
        &mut clicker::ClickFlash,
    )>,
) {
    let own = own.into_inner();
    let tick = time.into_inner().delta_secs();
    for (entity, owner, mut sprite, mut flash) in &mut flashing {
        let base = if **owner == *own {
            Color::srgb(0.2, 0.7, 0.3)
        } else {
            clicker::color_for(**owner)
        };
        let next = (**flash - tick).max(0.0);
        if next <= 0.0 {
            sprite.color = base;
            commands.entity(entity).remove::<clicker::ClickFlash>();
        } else {
            **flash = next;
            sprite.color = base.mix(&Color::WHITE, next / clicker::FLASH_SECS);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::animate_flash;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(net_id::NetworkId(1));
        let target = app
            .world_mut()
            .spawn((
                clicker::Owner(net_id::NetworkId(1)),
                Sprite::from_color(Color::srgb(0.2, 0.7, 0.3), Vec2::splat(10.0)),
                clicker::ClickFlash(clicker::FLASH_SECS),
            ))
            .id();
        app.add_systems(Update, animate_flash);
        app.update();
        let sprite = app.world().get::<Sprite>(target).unwrap();
        assert_ne!(
            sprite.color,
            Color::srgb(0.2, 0.7, 0.3),
            "flash should whiten the sprite while active"
        );
        **app
            .world_mut()
            .get_mut::<clicker::ClickFlash>(target)
            .unwrap() = 0.0;
        app.update();
        assert!(app.world().get::<clicker::ClickFlash>(target).is_none());
        let sprite = app.world().get::<Sprite>(target).unwrap();
        assert_eq!(sprite.color, Color::srgb(0.2, 0.7, 0.3));
    }
}
