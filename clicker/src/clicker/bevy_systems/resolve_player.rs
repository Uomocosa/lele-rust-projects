use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::clicker;

pub fn resolve_player(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    peers: Query<(Entity, &clicker::Owner, Option<&clicker::PlayerNo>), With<clicker::CursorIcon>>,
    mut spots: Query<
        (&clicker::Owner, &mut Transform, &mut clicker::TargetPos),
        With<clicker::CursorIcon>,
    >,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    own: Res<net_id::NetworkId>,
) {
    let own = own.into_inner();
    let mut rest = Vec::new();
    for event in events.take_all() {
        // Identity comes only from gossip: request_response echoes our own
        // advisory back with the recipient as sender, which would mislabel
        // remotes, while gossip delivers exactly what the publisher sent.
        let p2p::Event::Gossip { from, data, .. } = &event else {
            rest.push(event);
            continue;
        };
        let Ok(msg) = bincode::deserialize::<clicker::CursorMsg>(data) else {
            rest.push(event);
            continue;
        };
        let clicker::CursorMsg::Move { owner, .. } = msg else {
            rest.push(event);
            continue;
        };
        let sender = net_id::NetworkId::from_peer(from);
        let claimed = owner;
        if sender == *own || *claimed == 0 {
            rest.push(event);
            continue;
        }
        let player = net_id::NetworkId(*claimed);
        for (entity, owner, numbered) in &peers {
            if ***owner != *sender || numbered.is_some() {
                continue;
            }
            let spot = clicker::spawn_spot(player);
            let base = clicker::color_for(player);
            commands.entity(entity).insert(clicker::PlayerNo(*claimed));
            commands.entity(entity).insert(clicker::CursorColor(base));
            commands
                .entity(entity)
                .insert(MeshMaterial2d(materials.add(base)));
            for (spot_owner, mut transform, mut target) in &mut spots {
                if ***spot_owner != *sender {
                    continue;
                }
                transform.translation.x = spot.x;
                transform.translation.y = spot.y;
                **target = spot;
                break;
            }
            tracing::info!(
                "cursor resolved peer={from} player={} hue={:.1}",
                *claimed,
                clicker::hue_for(player)
            );
            break;
        }
        rest.push(event);
    }
    events.extend(rest);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::resolve_player;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(net_id::NetworkId(99));
        let sender = net_id::NetworkId::from_peer("peer");
        let visual = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::Owner(sender),
                clicker::TargetPos(Vec2::ZERO),
                Transform::default(),
            ))
            .id();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: clicker::CursorMsg::Click {
                    owner: net_id::NetworkId(5),
                    delta: 1,
                },
            });
        app.add_systems(Update, resolve_player);
        app.update();
        assert!(app.world().get::<clicker::PlayerNo>(visual).is_none());
        let msg = clicker::CursorMsg::Move {
            owner: net_id::NetworkId(5),
            pos: [30.0, 40.0],
        };
        let data = bincode::serialize(&msg).unwrap_or_default();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Gossip {
                topic: "clicker/alpha/pos".to_string(),
                from: "peer".to_string(),
                data,
            });
        app.update();
        let numbered = app.world().get::<clicker::PlayerNo>(visual).unwrap();
        assert_eq!(**numbered, 5);
        assert_eq!(
            app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .len(),
            2
        );
    }

    #[test]
    fn resolve_survives_absorb_first() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(99));
        let sender = net_id::NetworkId::from_peer("peer");
        let visual = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::Owner(sender),
                clicker::TargetPos(Vec2::ZERO),
                Transform::default(),
            ))
            .id();
        let msg = clicker::CursorMsg::Move {
            owner: net_id::NetworkId(5),
            pos: [30.0, 40.0],
        };
        let data = bincode::serialize(&msg).unwrap_or_default();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Gossip {
                topic: "clicker/alpha/pos".to_string(),
                from: "peer".to_string(),
                data,
            });
        app.add_systems(
            Update,
            (clicker::bevy_systems::absorb_gossip, resolve_player).chain(),
        );
        app.update();
        let numbered = app.world().get::<clicker::PlayerNo>(visual).unwrap();
        assert_eq!(**numbered, 5);
    }

    #[test]
    fn resolve_then_absorb_applies_live_pos() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(99));
        let sender = net_id::NetworkId::from_peer("peer");
        let visual = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::Owner(sender),
                clicker::TargetPos(Vec2::ZERO),
                Transform::default(),
            ))
            .id();
        let msg = clicker::CursorMsg::Move {
            owner: net_id::NetworkId(5),
            pos: [30.0, 40.0],
        };
        let data = bincode::serialize(&msg).unwrap_or_default();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Gossip {
                topic: "clicker/alpha/pos".to_string(),
                from: "peer".to_string(),
                data,
            });
        app.add_systems(
            Update,
            (resolve_player, clicker::bevy_systems::absorb_gossip).chain(),
        );
        app.update();
        let numbered = app.world().get::<clicker::PlayerNo>(visual).unwrap();
        assert_eq!(**numbered, 5);
        let target = app.world().get::<clicker::TargetPos>(visual).unwrap();
        assert!((target.x - 30.0).abs() < 0.001);
        assert!((target.y - 40.0).abs() < 0.001);
        assert!(
            !app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .is_empty()
        );
    }
}
