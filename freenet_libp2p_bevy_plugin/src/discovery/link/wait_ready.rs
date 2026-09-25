use crate::p2p;

use std::time::Duration;

use super::super::constants;

pub async fn wait_ready(
    ready: &mut tokio::sync::watch::Receiver<Option<p2p::Ready>>,
) -> Option<(String, Vec<String>)> {
    let deadline = tokio::time::Instant::now()
        .checked_add(Duration::from_secs(constants::READY_TIMEOUT_SECS))?;
    loop {
        let value = ready.borrow().clone();
        if let Some(info) = value {
            return Some((info.peer_id, info.addrs));
        }
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return None;
        }
        if tokio::time::timeout(remaining, ready.changed())
            .await
            .is_err()
        {
            return ready.borrow().clone().map(|r| (r.peer_id, r.addrs));
        }
    }
}

// no test_usage necessary
