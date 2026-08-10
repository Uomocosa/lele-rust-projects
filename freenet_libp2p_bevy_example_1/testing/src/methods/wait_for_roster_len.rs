use std::time::Duration;

use bevy_freenet::roster;

use crate::methods::recv_roster_notification;

pub async fn wait_for_roster_len(
    client: &mut bevy_freenet::freenet::FreenetClient,
    expected_len: usize,
    timeout: Duration,
) -> Option<roster::RosterState> {
    let deadline = tokio::time::Instant::now() + timeout;
    while tokio::time::Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if let Some(entries) = recv_roster_notification(client, remaining).await
            && entries.len() >= expected_len
        {
            return Some(entries);
        }
    }
    None
}
