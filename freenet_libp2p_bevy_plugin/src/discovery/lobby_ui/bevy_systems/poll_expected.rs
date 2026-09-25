#![allow(clippy::needless_pass_by_value)]
use std::collections::BTreeSet;

use bevy::prelude::*;

use super::super::super::session::expected_rx::ExpectedRx;
use super::super::super::session::join_clock::JoinClock;
use super::super::super::session::join_gate::JoinGate;

pub fn poll_expected(
    feed: Res<ExpectedRx>,
    mut gate: ResMut<JoinGate>,
    mut clock: ResMut<JoinClock>,
) {
    let Ok(mut guard) = feed.lock() else {
        return;
    };
    let Some(rx) = guard.as_mut() else {
        return;
    };
    let mut latest: Option<Vec<String>> = None;
    while let Ok(peers) = rx.try_recv() {
        latest = Some(peers);
    }
    let Some(peers) = latest else {
        return;
    };
    let incoming: BTreeSet<String> = peers.into_iter().collect();
    let grown = gate
        .expected
        .as_ref()
        .is_none_or(|old| incoming.iter().any(|peer| !old.contains(peer)));
    gate.expected = Some(incoming);
    if grown {
        clock.last_new_peer = Some(std::time::Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::poll_expected;
    use crate::discovery;
    use bevy::prelude::*;
    use std::sync::Mutex;

    #[test]
    fn test_usage() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<String>>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(discovery::session::ExpectedRx(Mutex::new(Some(rx))));
        app.insert_resource(discovery::session::JoinGate::default());
        app.insert_resource(discovery::session::JoinClock::default());
        tx.send(vec!["peer-2".to_string(), "peer-3".to_string()])
            .ok();
        app.add_systems(Update, poll_expected);
        app.update();
        let gate = app.world().resource::<discovery::session::JoinGate>();
        assert!(
            gate.expected
                .as_ref()
                .is_some_and(|set| { set.contains("peer-2") && set.contains("peer-3") })
        );
        assert!(
            app.world()
                .resource::<discovery::session::JoinClock>()
                .last_new_peer
                .is_some()
        );
    }

    #[test]
    fn latest_snapshot_wins() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<String>>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(discovery::session::ExpectedRx(Mutex::new(Some(rx))));
        app.insert_resource(discovery::session::JoinGate::default());
        app.insert_resource(discovery::session::JoinClock::default());
        tx.send(vec!["peer-2".to_string()]).ok();
        tx.send(vec!["peer-4".to_string()]).ok();
        app.add_systems(Update, poll_expected);
        app.update();
        let gate = app.world().resource::<discovery::session::JoinGate>();
        assert!(
            gate.expected
                .as_ref()
                .is_some_and(|set| { set.len() == 1 && set.contains("peer-4") })
        );
    }
}
