use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::clicker;

const INTERVAL_SECS: f64 = 1.0 / 15.0;
const HEARTBEAT_SECS: f64 = 5.0;
const EPSILON_SQ: f32 = 4.0;

pub fn publish_cursor(
    time: Res<Time>,
    mut last: Local<f64>,
    mut last_pos: Local<Option<Vec2>>,
    cursors: Query<(&clicker::Owner, &Transform), With<clicker::CursorIcon>>,
    own: Res<net_id::NetworkId>,
    lobby: Res<clicker::ActiveLobby>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
) {
    let time = time.into_inner();
    let own = own.into_inner();
    let lobby = lobby.into_inner();
    let commands = commands.into_inner();
    let now = time.elapsed().as_secs_f64();
    let primed = last_pos.is_some();
    if primed && now - *last < INTERVAL_SECS {
        return;
    }
    let mut here = None;
    for (owner, transform) in &cursors {
        if ***owner == **own {
            here = Some(Vec2::new(transform.translation.x, transform.translation.y));
            break;
        }
    }
    let Some(pos) = here else {
        return;
    };
    let moved = last_pos.is_some_and(|prev| prev.distance_squared(pos) >= EPSILON_SQ);
    let heartbeat_due = now - *last >= HEARTBEAT_SECS;
    if primed && !moved && !heartbeat_due {
        return;
    }
    *last = now;
    *last_pos = Some(pos);
    let msg = clicker::CursorMsg::Move {
        owner: *own,
        pos: [pos.x, pos.y],
    };
    let data = bincode::serialize(&msg).unwrap_or_default();
    if data.is_empty() {
        return;
    }
    commands.push(p2p::Command::Publish {
        topic: clicker::pos_topic(lobby),
        data,
    });
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::publish_cursor;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.world_mut().spawn((
            clicker::CursorIcon,
            clicker::Owner(net_id::NetworkId(1)),
            Transform::from_translation(Vec3::new(10.0, 20.0, 10.0)),
        ));
        app.add_systems(Update, publish_cursor);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
        let Some(p2p::Command::Publish { topic, data }) = commands.first() else {
            panic!("expected Publish");
        };
        assert_eq!(topic, "clicker/alpha/pos");
        let decoded: Option<clicker::CursorMsg> = bincode::deserialize(data).ok();
        assert!(matches!(decoded, Some(clicker::CursorMsg::Move { .. })));
    }
}
