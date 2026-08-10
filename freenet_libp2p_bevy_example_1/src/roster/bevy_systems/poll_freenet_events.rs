use bevy::prelude::*;

use crate::roster;

pub fn poll_freenet_events(
    mut events: ResMut<roster::RosterEvents>,
    mut roster: ResMut<roster::Roster>,
) {
    while let Ok(event) = events.try_recv() {
        match event {
            roster::Event::Roster { entries } => **roster = entries,
            roster::Event::ConnectionError(reason) => {
                tracing::error!(target: "roster", reason, "freenet connection error");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::poll_freenet_events;
    use crate::boxes;
    use crate::roster;

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
                updated_at: 1,
            },
        );
        tx.send(roster::Event::Roster {
            entries: entries.clone(),
        })
        .unwrap();

        app.insert_resource(roster::RosterEvents(rx));
        app.insert_resource(roster::Roster::default());
        app.add_systems(Update, poll_freenet_events);
        app.update();

        let roster = app.world().resource::<roster::Roster>();
        assert_eq!(**roster, entries);
    }
}
