use crate::discovery;

pub async fn refresh(
    lobby_client: &mut discovery::freenet::LobbyClient,
) -> Result<discovery::Lobby, discovery::Error> {
    let instance_id = *lobby_client.key.id();
    let fresh = discovery::freenet::fetch(&mut lobby_client.client, instance_id).await?;
    let cached = std::mem::take(&mut lobby_client.lobby);
    lobby_client.lobby = discovery::freenet::merge_lobby(cached, fresh);
    Ok(lobby_client.lobby.clone())
}

// no test_usage necessary
