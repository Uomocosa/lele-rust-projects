use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

use crate::roster;

pub fn poll_freenet_events(
    mut events: ResMut<roster::RosterEvents>,
    mut roster: ResMut<roster::Roster>,
    mut status: ResMut<roster::FreenetStatus>,
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
                **roster = roster::prune_stale(entries, now, roster::ROSTER_ENTRY_TTL_SECS);
                *status = roster::FreenetStatus::Connected;
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
    use crate::boxes;
    use crate::roster;

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let mut entries = roster::RosterState::default();
        entries.insert(
            boxes::PlayerId(1),
            roster::PeerEntry {
                peer_id: "peer-1".to_string(),
                addrs: vec![],
                updated_at: now(),
            },
        );
        tx.send(roster::Event::Roster {
            entries: entries.clone(),
        })
        .unwrap();

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
}
