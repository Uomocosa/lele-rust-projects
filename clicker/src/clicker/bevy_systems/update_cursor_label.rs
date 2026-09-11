use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

pub fn update_cursor_label(
    own: Res<net_id::NetworkId>,
    targets: Query<(&clicker::Owner, &clicker::ClickCounter)>,
    mut labels: Query<&mut Text2d, With<clicker::CursorLabel>>,
) {
    let own = own.into_inner();
    let mut mine = 0;
    for (owner, counter) in &targets {
        if **owner == *own {
            mine = **counter;
            break;
        }
    }
    let text = clicker::math_formatter(mine);
    for mut label in &mut labels {
        (**label).clone_from(&text);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::update_cursor_label;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter(1_234),
        ));
        let label = app
            .world_mut()
            .spawn((clicker::CursorLabel, Text2d::new("0")))
            .id();
        app.add_systems(Update, update_cursor_label);
        app.update();
        let text = app.world().get::<Text2d>(label).unwrap();
        assert_eq!(text.0, "1.2e+3");
    }
}
