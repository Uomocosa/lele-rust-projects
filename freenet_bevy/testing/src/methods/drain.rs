use std::time::Duration;

pub async fn drain(client: &mut freenet_bevy::freenet::FreenetClient) {
    while let Some(Ok(_)) = client.recv_timeout(Duration::from_millis(50)).await {}
}
