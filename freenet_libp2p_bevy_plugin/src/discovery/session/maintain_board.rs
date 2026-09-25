use std::time::Instant;

use super::super::Event;
use super::super::id::now_epoch;
use super::super::lobby_rooms::{IndexClient, poll, publish_presence};
use super::super::timing::Timing;
use super::session::Session;

pub async fn maintain_board(
    session: &mut Session,
    index: &mut IndexClient,
    timing: &Timing,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    if let Ok(board) = poll(index).await {
        session.catalogue = board;
        let _ = events.send(Event::CatalogueChanged);
    }
    let now = Instant::now();
    let due = session
        .last_republish
        .is_none_or(|last| now.duration_since(last).as_secs() >= timing.republish_secs);
    if due && let Some(room) = &session.room {
        let _ = publish_presence(
            index,
            &room.name,
            &session.me,
            &session.addrs,
            now_epoch(),
            session.capacity,
        );
        session.last_republish = Some(now);
    }
}

// no test_usage necessary
