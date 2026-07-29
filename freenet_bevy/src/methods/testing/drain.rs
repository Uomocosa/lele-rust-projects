use std::time::Duration;

pub async fn drain(client: &mut crate::structs::freenet_client::FreenetClient) {
    while let Some(Ok(_)) = client.recv_timeout(Duration::from_millis(50)).await {}
}
