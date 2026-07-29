use tokio_tungstenite::tungstenite::Message;

pub async fn send(
    client: &crate::structs::freenet_client::FreenetClient,
    request: freenet_stdlib::client_api::ClientRequest<'_>,
) -> Result<(), crate::ClientError> {
    let bytes = bincode::serialize(&request)?;
    client
        .write
        .send(Message::Binary(bytes.into()))
        .map_err(|_| crate::ClientError::SendError)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
