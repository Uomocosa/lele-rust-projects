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
        let taken = ctx
            .peers
            .iter()
            .any(|(_, _, numbered, _)| numbered.is_some_and(|number| **number == *claimed));
        if taken {
            rest.push(event);
            continue;
        }
        let pending_reveal = ctx
            .gate
            .pending
            .iter()
            .find(|held| held.peer == *from)
            .map(|held| (held.reveal_at, held.fail_at));
        for (entity, owner, numbered, _marker) in &ctx.peers {
            if ***owner != *sender || numbered.is_some() {
                continue;
            }
            let spot = clicker::spawn_spot(player);
            if let Some((reveal_at, fail_at)) = pending_reveal {
                let now = std::time::Instant::now();
                let reveal_at = if now >= reveal_at { now } else { reveal_at };
                ctx.commands.entity(entity).insert(clicker::PendingReveal {
                    reveal_at,
                    fail_at,
                    player: Some(*claimed),
                });
            } else {
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
    use crate::lobby;
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
        app.insert_resource(lobby::JoinGate::default());
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
        app.insert_resource(lobby::JoinGate::default());
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
        app.insert_resource(lobby::JoinGate::default());
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
    fn duplicate_player_no_never_labels_twice() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(99));
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(lobby::JoinGate::default());
        let sender = net_id::NetworkId::from_peer("peer");
        app.world_mut().spawn((
            clicker::CursorIcon,
            clicker::Owner(sender),
            clicker::PlayerNo(5),
            clicker::ClickCounter(5),
            clicker::TargetPos(Vec2::ZERO),
            Transform::default(),
        ));
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
        app.add_systems(Update, resolve_player);
        app.update();
        assert!(
            app.world().get::<clicker::PlayerNo>(visual).is_none(),
            "second slot for a taken player number stays unlabeled"
        );
    }

    #[test]
    fn armed_pending_defers_label_to_reveal() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(99));
        app.insert_resource(clicker::ScoreTombstones::default());
        let now = std::time::Instant::now();
        let mut gate = lobby::JoinGate::default();
        gate.arm_pending(lobby::PendingJoin {
            peer: "peer".to_string(),
            joiner: net_id::NetworkId::from_peer("peer"),
            reveal_at: now,
            fail_at: now
                .checked_add(std::time::Duration::from_secs(30))
                .unwrap_or(now),
        });
        app.insert_resource(gate);
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
        app.add_systems(Update, resolve_player);
        app.update();
        let marker = app.world().get::<clicker::PendingReveal>(visual);
        assert_eq!(
            marker.map(|mark| mark.player),
            Some(Some(5)),
            "armed pending defers the claimed identity to the reveal path"
        );
        assert!(
            app.world().get::<clicker::PlayerNo>(visual).is_none(),
            "no direct label while the reveal path owns the peer"
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
        app.insert_resource(lobby::JoinGate::default());
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
