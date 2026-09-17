use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;
use crate::lobby;

pub fn absorb_sync(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    mut targets: Query<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    mut pending: ResMut<clicker::PendingClicks>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
    own: Res<net_id::NetworkId>,
    mut ctx: clicker::bevy_systems::SyncCtx,
) {
    let own = *own.into_inner();
    let commands = commands.into_inner();
    let mut rest = Vec::new();
    for event in events.take_all() {
        match event {
            p2p::Event::Message { from, payload } => match payload {
                clicker::CursorMsg::SyncReq { requester } => {
                    if requester == own {
                        continue;
                    }
                    let entries = ctx.snapshot_entries(&targets);
                    commands.push(p2p::Command::Send {
                        peer_id: from,
                        payload: clicker::CursorMsg::SyncAck {
                            target: requester,
                            entries,
                        },
                    });
                }
                clicker::CursorMsg::SyncAck { target, entries } => {
                    if target != own {
                        continue;
                    }
                    if !ctx
                        .gate
                        .synced
                        .iter()
                        .any(|entry| entry.as_str() == from.as_str())
                    {
                        ctx.gate.synced.push(lobby::SyncedPeer(from.clone()));
                    }
                    let sender = net_id::NetworkId::from_peer(&from);
                    merge_entries(&mut targets, &mut pending, sender, entries);
                }
                clicker::CursorMsg::PexAsk { .. } | clicker::CursorMsg::PexResp { .. } => {}
                other => rest.push(p2p::Event::Message {
                    from,
                    payload: other,
                }),
            },
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

// needed helper: max-merges sync entries, parking unknown slots
fn merge_entries(
    targets: &mut Query<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    pending: &mut clicker::PendingClicks,
    sender: net_id::NetworkId,
    entries: Vec<(net_id::NetworkId, i32)>,
) {
    for (id, count) in entries {
        let mut done = false;
        for (slot_owner, player, mut counter) in targets.iter_mut() {
            if ***slot_owner == *id || player.is_some_and(|p| **p == *id) {
                let current = **counter;
                if count > current {
                    counter.add(count.saturating_sub(current));
                }
                done = true;
                break;
            }
        }
        if !done {
            pending.items.push(clicker::PendingClick {
                sender,
                owner: id,
                delta: count,
                absolute: true,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::absorb_sync;
    use crate::clicker;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::PlayerNo(1),
            clicker::ClickCounter(2),
        ));
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: clicker::CursorMsg::SyncReq {
                    requester: net_id::NetworkId(2),
                },
            });
        app.add_systems(Update, absorb_sync);
        app.update();
        assert_eq!(
            app.world()
                .resource::<p2p::Commands<clicker::CursorMsg>>()
                .len(),
            1
        );
    }

    #[test]
    fn reply_carries_tombstone() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(9)),
            clicker::PlayerNo(9),
            clicker::ClickCounter(2),
        ));
        app.world_mut()
            .resource_mut::<clicker::ScoreTombstones>()
            .keep(9, 5);
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: clicker::CursorMsg::SyncReq {
                    requester: net_id::NetworkId(2),
                },
            });
        app.add_systems(Update, absorb_sync);
        app.update();
        let entries = app
            .world()
            .resource::<p2p::Commands<clicker::CursorMsg>>()
            .iter()
            .filter_map(|command| match command {
                p2p::Command::Send {
                    payload: clicker::CursorMsg::SyncAck { entries, .. },
                    ..
                } => Some(entries.clone()),
                _ => None,
            })
            .collect::<Vec<Vec<(net_id::NetworkId, i32)>>>();
        assert!(
            entries
                .iter()
                .any(|list| list.contains(&(net_id::NetworkId(9), 5))),
            "sync reply carries retained scores, got {entries:?}"
        );
    }

    #[test]
    fn reply_omits_never_seen_tombstone() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut()
            .resource_mut::<clicker::ScoreTombstones>()
            .keep(9, 5);
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: clicker::CursorMsg::SyncReq {
                    requester: net_id::NetworkId(2),
                },
            });
        app.add_systems(Update, absorb_sync);
        app.update();
        let entries = app
            .world()
            .resource::<p2p::Commands<clicker::CursorMsg>>()
            .iter()
            .filter_map(|command| match command {
                p2p::Command::Send {
                    payload: clicker::CursorMsg::SyncAck { entries, .. },
                    ..
                } => Some(entries.clone()),
                _ => None,
            })
            .collect::<Vec<Vec<(net_id::NetworkId, i32)>>>();
        assert!(
            entries
                .iter()
                .all(|list| list.iter().all(|(id, _)| *id != net_id::NetworkId(9))),
            "retained scores without local provenance never ride the reply, got {entries:?}"
        );
    }

    #[test]
    fn test_ack_merges() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(net_id::NetworkId(1));
        let target = app
            .world_mut()
            .spawn((
                clicker::Owner(net_id::NetworkId::from_peer("peer")),
                clicker::PlayerNo(2),
                clicker::ClickCounter(1),
            ))
            .id();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: clicker::CursorMsg::SyncAck {
                    target: net_id::NetworkId(1),
                    entries: vec![(net_id::NetworkId(2), 7)],
                },
            });
        app.add_systems(Update, absorb_sync);
        app.update();
        assert_eq!(
            **app.world().get::<clicker::ClickCounter>(target).unwrap(),
            7
        );
    }

    #[test]
    fn ack_sender_recorded_as_synced() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer-2".to_string(),
                payload: clicker::CursorMsg::SyncAck {
                    target: net_id::NetworkId(1),
                    entries: vec![(net_id::NetworkId(2), 7)],
                },
            });
        app.add_systems(Update, absorb_sync);
        app.update();
        assert!(
            app.world()
                .resource::<lobby::JoinGate>()
                .synced
                .iter()
                .any(|entry| entry.as_str() == "peer-2"),
            "merged ack marks its sender synced"
        );
    }
}
