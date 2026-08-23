use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

use crate::net_id;
use crate::roster;

pub fn poll_freenet_events(
    mut events: ResMut<roster::RosterEvents>,
    mut roster: ResMut<roster::Roster>,
    mut status: ResMut<roster::FreenetStatus>,
    mut joined_tx: MessageWriter<roster::PeerJoined>,
    mut left_tx: MessageWriter<roster::PeerLeft>,
    identity: Res<net_id::LocalIdentity>,
) {
    while let Ok(event) = events.try_recv() {
        match event {
            roster::Event::Connecting { attempt } => {
                *status = if attempt == 1 {
                    roster::FreenetStatus::Connecting
                } else {
                    roster::FreenetStatus::Retrying { attempt }
                };
            }
            roster::Event::Roster { entries } => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let previous: HashSet<net_id::NetworkId> = roster.keys().copied().collect();
                let merged = roster::merge_roster(std::mem::take(&mut **roster), entries);
                **roster = roster::prune_stale(merged, now, roster::ROSTER_ENTRY_TTL_SECS);
                *status = roster::FreenetStatus::Connected;

                let current: HashSet<net_id::NetworkId> = roster.keys().copied().collect();
                for id in current.difference(&previous) {
                    if *id != **identity {
                        joined_tx.write(roster::PeerJoined(*id));
                    }
                }
                for id in previous.difference(&current) {
                    if *id != **identity {
                        left_tx.write(roster::PeerLeft(*id));
                    }
                }
            }
            roster::Event::ConnectionError(reason) => {
                tracing::error!(target: "roster", reason, "freenet connection error");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use bevy::prelude::*;

    use super::poll_freenet_events;
    use crate::net_id;
    use crate::roster;

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn entry(peer_id: &str) -> roster::PeerEntry {
        roster::PeerEntry {
            peer_id: peer_id.to_string(),
            addrs: vec![],
            updated_at: now(),
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<roster::PeerJoined>();
        app.add_message::<roster::PeerLeft>();
        app.insert_resource(net_id::LocalIdentity(net_id::NetworkId(99)));
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<roster::Event>();

        let mut entries = roster::RosterState::default();
        entries.insert(net_id::NetworkId(1), entry("peer-1"));
        tx.send(roster::Event::Roster {
            entries: entries.clone(),
        })
        .ok();

        app.insert_resource(roster::RosterEvents(rx));
        app.insert_resource(roster::Roster::default());
        app.insert_resource(roster::FreenetStatus::default());
        app.add_systems(Update, poll_freenet_events);
        app.update();

        let roster = app.world().resource::<roster::Roster>();
        assert_eq!(**roster, entries);
        let status = app.world().resource::<roster::FreenetStatus>();
        assert_eq!(*status, roster::FreenetStatus::Connected);
    }

    #[test]
    fn emits_join_on_new_peer_and_nothing_on_stable_roster() {
        let mut app = App::new();
        app.add_message::<roster::PeerJoined>();
        app.add_message::<roster::PeerLeft>();
        app.insert_resource(net_id::LocalIdentity(net_id::NetworkId(99)));
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<roster::Event>();

        let mut initial = roster::RosterState::default();
        initial.insert(net_id::NetworkId(1), entry("peer-1"));
        initial.insert(net_id::NetworkId(99), entry("self"));
        tx.send(roster::Event::Roster {
            entries: initial.clone(),
        })
        .ok();
        app.insert_resource(roster::RosterEvents(rx));
        app.insert_resource(roster::Roster::default());
        app.insert_resource(roster::FreenetStatus::default());
        app.add_systems(Update, poll_freenet_events);
        app.update();

        let joined: Vec<_> = {
            let mut joined_reader = app
                .world_mut()
                .resource_mut::<bevy::ecs::message::Messages<roster::PeerJoined>>();
            let mut seen: Vec<_> = joined_reader.drain().collect();
            seen.sort_by_key(|e| e.0);
            seen
        };
        assert_eq!(joined, vec![roster::PeerJoined(net_id::NetworkId(1))]);

        tx.send(roster::Event::Roster {
            entries: initial.clone(),
        })
        .ok();
        app.update();

        let mut joined_reader = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<roster::PeerJoined>>();
        let joined: Vec<_> = joined_reader.drain().collect();
        assert!(joined.is_empty(), "self is filtered, peer known already");

        let mut left_reader = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<roster::PeerLeft>>();
        let left: Vec<_> = left_reader.drain().collect();
        assert!(left.is_empty(), "no peer left on a stable roster");
    }
}
