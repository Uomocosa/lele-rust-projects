use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p};

use super::resolve_ctx;
use crate::clicker;

pub fn resolve_player(mut ctx: resolve_ctx::ResolveCtx) {
    let topic = ctx.topic();
    let own = ctx.own.into_inner();
    let mut rest = Vec::new();
    for event in ctx.events.take_all() {
        // Identity comes only from gossip: request_response echoes our own
        // advisory back with the recipient as sender, which would mislabel
        // remotes, while gossip delivers exactly what the publisher sent.
        let p2p::Event::Gossip {
            topic: incoming,
            from,
            data,
        } = &event
        else {
            rest.push(event);
            continue;
        };
        if *incoming != topic {
            rest.push(event);
            continue;
        }
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
        for (entity, owner, numbered) in &ctx.peers {
            if ***owner != *sender || numbered.is_some() {
                continue;
            }
            let spot = clicker::spawn_spot(player);
            clicker::label_slot(
                &mut ctx.commands,
                &mut ctx.materials,
                entity,
                from,
                *claimed,
            );
            if let Some(saved) = ctx.tombstones.restore(*claimed)
                && let Ok(mut counter) = ctx.counters.get_mut(entity)
            {
                counter.max(saved);
            }
            for (spot_owner, mut transform, mut target) in &mut ctx.spots {
                if ***spot_owner != *sender {
                    continue;
                }
                transform.translation.x = spot.x;
                transform.translation.y = spot.y;
                **target = spot;
                break;
            }
            break;
        }
        rest.push(event);
    }
    ctx.events.extend(rest);
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
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(clicker::ScoreTombstones::default());
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
        app.insert_resource(clicker::ScoreTombstones::default());
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
        app.insert_resource(clicker::ScoreTombstones::default());
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

    #[test]
    fn test_foreign_topic_never_resolves() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(99));
        app.insert_resource(clicker::ScoreTombstones::default());
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
        let roster = clicker::gossip_roster_topic(&clicker::ActiveLobby("alpha".to_string()));
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Gossip {
                topic: roster,
                from: "peer".to_string(),
                data: vec![3u8, 0, 0, 0, 0, 0, 0, 0],
            });
        app.add_systems(Update, resolve_player);
        app.update();
        assert!(app.world().get::<clicker::PlayerNo>(visual).is_none());
    }
}
