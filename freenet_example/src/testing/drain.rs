use std::time::Duration;

use crate::FreenetClient;

#[allow(dead_code)]
pub async fn drain(client: &mut FreenetClient) {
    while let Some(_) = client.recv_timeout(Duration::from_millis(50)).await {}
}
