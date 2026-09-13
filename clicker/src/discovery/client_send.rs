use tokio_tungstenite::tungstenite::Message;

use freenet_stdlib::client_api::ClientRequest;

use crate::discovery;

pub fn send(
    client: &discovery::Client,
    request: &ClientRequest<'_>,
) -> Result<(), discovery::Error> {
    let bytes = bincode::serialize(&request)?;
    client
        .write
        .send(Message::Binary(bytes.into()))
        .map_err(|_| discovery::Error::ChannelSend)?;
    Ok(())
}
// no test_usage necessary
