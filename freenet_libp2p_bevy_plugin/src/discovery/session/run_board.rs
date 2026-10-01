use std::time::{Duration, Instant};

use crate::discovery;
use discovery::Timing;
use discovery::basic::constants;
use discovery::id::{RemotePeerId, now_epoch};
use discovery::lobby_rooms::{IndexClient, RoomCatalogue, poll, publish_presence, refresh};
use discovery::session::BoardTarget;

pub async fn run_board(
    mut index: IndexClient,
    me: RemotePeerId,
    capacity: u16,
    timing: Timing,
    mut target: tokio::sync::watch::Receiver<Option<BoardTarget>>,
    boards: tokio::sync::mpsc::UnboundedSender<RoomCatalogue>,
) {
    let mut tick = tokio::time::interval(Duration::from_secs(timing.tick_secs.max(1)));
    let mut last_sent: Option<RoomCatalogue> = None;
    let mut last_refresh: Option<Instant> = None;
    let mut last_publish: Option<Instant> = None;
    loop {
        tokio::select! {
            _ = tick.tick() => {}
            changed = target.changed() => {
                if changed.is_err() {
                    return;
                }
                last_publish = None;
            }
        }
        let now = Instant::now();
        let mut board = poll(&mut index).await.ok();
        let refresh_due =
            last_refresh.is_none_or(|last| now.duration_since(last).as_secs() >= timing.board_secs);
        if refresh_due {
            let timeout = Duration::from_secs(constants::BOARD_REQUEST_TIMEOUT_SECS);
            if let Ok(Ok(fresh)) = tokio::time::timeout(timeout, refresh(&mut index)).await {
                board = Some(fresh);
            }
            last_refresh = Some(now);
        }
        if let Some(board) = board
            && last_sent.as_ref() != Some(&board)
        {
            last_sent = Some(board.clone());
            if boards.send(board).is_err() {
                return;
            }
        }
        let publish_due = last_publish
            .is_none_or(|last| now.duration_since(last).as_secs() >= timing.republish_secs);
        let current = target.borrow_and_update().clone();
        if publish_due && let Some(current) = current {
            let _ = publish_presence(
                &mut index,
                &current.room,
                &me,
                &current.addrs,
                now_epoch(),
                capacity,
            );
            last_publish = Some(now);
        }
    }
}

// no test_usage necessary
