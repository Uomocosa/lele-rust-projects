use std::time::Duration;

pub async fn connect(
    port: u16,
) -> Result<
    freenet_libp2p_bevy_example_2_lib::freenet::FreenetClient,
    freenet_libp2p_bevy_example_2_lib::freenet::FreenetConnectionError,
> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        match freenet_libp2p_bevy_example_2_lib::freenet::FreenetClient::connect("127.0.0.1", port)
            .await
        {
            Ok(client) => return Ok(client),
            Err(e) if tokio::time::Instant::now() > deadline => return Err(e),
            _ => tokio::time::sleep(Duration::from_millis(200)).await,
        }
    }
}
