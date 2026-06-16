use tokio_tungstenite::tungstenite::Message;

use freenet_stdlib::client_api::ClientRequest;

use crate::ClientError as Ce;

pub async fn send(client: &crate::FreenetClient, request: ClientRequest<'_>) -> Result<(), Ce> {
    let bytes = bincode::serialize(&request)?;
    client
        .write
        .send(Message::Binary(bytes.into()))
        .map_err(|_| Ce::ChannelSend)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
