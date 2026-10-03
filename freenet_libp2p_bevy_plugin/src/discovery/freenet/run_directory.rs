use std::time::{Duration, Instant};

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;

use crate::discovery;
use discovery::freenet::basic::constants;

pub async fn run_directory(
    mut directory_client: discovery::freenet::DirectoryClient,
    me: discovery::PeerId,
    capacity: u16,
    timing: discovery::Timing,
    mut target: Receiver<Option<discovery::session::PublishTarget>>,
    directories: UnboundedSender<discovery::Directory>,
) {
    let mut tick = tokio::time::interval(Duration::from_secs(timing.tick_secs.max(1)));
    let mut last_sent: Option<discovery::Directory> = None;
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
        let mut directory = discovery::freenet::poll(&mut directory_client).await.ok();
        if is_due(last_refresh, now, timing.board_secs) {
            let timeout = Duration::from_secs(constants::REQUEST_TIMEOUT_SECS);
            let refresh = discovery::freenet::refresh(&mut directory_client);
            if let Ok(Ok(fresh)) = tokio::time::timeout(timeout, refresh).await {
                directory = Some(fresh);
            }
            last_refresh = Some(now);
        }
        if let Some(directory) = directory
            && last_sent.as_ref() != Some(&directory)
        {
            last_sent = Some(directory.clone());
            if directories.send(directory).is_err() {
                return;
            }
        }
        let current = target.borrow_and_update().clone();
        if is_due(last_publish, now, timing.republish_secs)
            && let Some(current) = current
        {
            let presence = discovery::Presence {
                addrs: current.addrs,
                updated_at: discovery::now_epoch(),
            };
            let _ = discovery::freenet::publish_presence(
                &mut directory_client,
                &current.room,
                &me,
                presence,
                capacity,
            );
            last_publish = Some(now);
        }
    }
}

// needed helper: true when `every_secs` have elapsed since `last` (or it never ran)
fn is_due(last: Option<Instant>, now: Instant, every_secs: u64) -> bool {
    last.is_none_or(|last| now.duration_since(last).as_secs() >= every_secs)
}

// no test_usage necessary
