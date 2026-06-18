use std::time::Duration;

use crate::FreenetClient;

pub async fn drain(client: &mut FreenetClient) {
    while client
        .recv_timeout(Duration::from_millis(50))
        .await
        .is_some()
    {}
}
