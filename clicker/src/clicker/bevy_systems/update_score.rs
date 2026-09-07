use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

pub fn update_score(
    own: Res<net_id::NetworkId>,
    global: Res<clicker::GlobalCounter>,
    targets: Query<(&clicker::Owner, &clicker::ClickCounter)>,
    mut texts: Query<(
        &mut Text2d,
        Option<&clicker::OwnScore>,
        Option<&clicker::GlobalScore>,
    )>,
) {
    let own = own.into_inner();
    let global = global.into_inner();
    let mut mine = 0;
    for (owner, counter) in &targets {
        if **owner == *own {
            mine = **counter;
            break;
        }
    }
    for (mut text, own_mark, global_mark) in &mut texts {
        if own_mark.is_some() {
            **text = format!("you: {mine}");
        } else if global_mark.is_some() {
            **text = format!("global: {}", **global);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::update_score;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(net_id::NetworkId(1));
        let mut global = clicker::GlobalCounter::default();
        global.add(7);
        app.insert_resource(global);
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter(3),
        ));
        app.world_mut().spawn((
            clicker::OwnScore,
            Text2d::new("you: 0"),
            Transform::default(),
        ));
        app.world_mut().spawn((
            clicker::GlobalScore,
            Text2d::new("global: 0"),
            Transform::default(),
        ));
        app.add_systems(Update, update_score);
        app.update();
        let mut found_own = false;
        let mut found_global = false;
        let mut query = app.world_mut().query::<(
            &Text2d,
            Option<&clicker::OwnScore>,
            Option<&clicker::GlobalScore>,
        )>();
        for (text, own, global) in query.iter(app.world()) {
            if own.is_some() && text.0 == "you: 3" {
                found_own = true;
            }
            if global.is_some() && text.0 == "global: 7" {
                found_global = true;
            }
        }
        assert!(found_own);
        assert!(found_global);
    }
}
