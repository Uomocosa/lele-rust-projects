use std::time::Duration;

use crate::FreenetClient;

pub async fn connect(port: u16) -> FreenetClient {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        match FreenetClient::connect("127.0.0.1", port).await {
            Ok(c) => return c,
            Err(e) => {
                if tokio::time::Instant::now() >= deadline {
                    panic!("could not connect within 15 s: {e}");
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}
