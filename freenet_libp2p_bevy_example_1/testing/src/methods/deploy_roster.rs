use bevy_freenet::{boxes, roster};

pub async fn deploy_roster(
    port: u16,
    wasm: &[u8],
    params: &[u8],
    own_id: boxes::PlayerId,
    own_entry: roster::PeerEntry,
) -> Result<(bevy_freenet::freenet::FreenetClient, roster::RosterState), String> {
    let (client, _key, entries) =
        roster::setup_contract("127.0.0.1", port, wasm, params, own_id, own_entry).await?;
    Ok((client, entries))
}
