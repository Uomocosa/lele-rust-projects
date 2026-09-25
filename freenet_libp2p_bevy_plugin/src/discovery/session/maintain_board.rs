use std::time::Instant;

use crate::discovery;
use discovery::Event;
use discovery::Timing;
use discovery::id::now_epoch;
use discovery::lobby_rooms::{IndexClient, poll, publish_presence, refresh};
use discovery::session::Session;

pub async fn maintain_board(
    session: &mut Session,
    index: &mut IndexClient,
    timing: &Timing,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    let _ = poll(index).await;
    let now = Instant::now();
    let refresh_due = session
        .last_board
        .is_none_or(|last| now.duration_since(last).as_secs() >= timing.board_secs);
    if refresh_due {
        if let Ok(board) = refresh(index).await {
            if board != session.catalogue {
                let _ = events.send(Event::CatalogueChanged);
            }
            session.catalogue = board;
        }
        session.last_board = Some(now);
    }
    let republish_due = session
        .last_republish
        .is_none_or(|last| now.duration_since(last).as_secs() >= timing.republish_secs);
    if republish_due && let Some(room) = &session.room {
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
