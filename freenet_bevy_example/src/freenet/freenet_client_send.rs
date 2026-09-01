use tokio_tungstenite::tungstenite::Message;

use crate::freenet;

pub async fn send(
    client: &freenet::FreenetClient,
    request: freenet_stdlib::client_api::ClientRequest<'_>,
) -> Result<(), freenet::FreenetConnectionError> {
    let bytes = bincode::serialize(&request)?;
    client
        .write
        .send(Message::Binary(bytes.into()))
        .map_err(|_| freenet::FreenetConnectionError::SendError)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
