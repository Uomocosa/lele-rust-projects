use std::collections::BTreeMap;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::{StateDelta, UpdateData};

use crate::discovery;
use crate::net_id;

pub fn publish_presence(
    lobby_client: &mut discovery::freenet::LobbyClient,
    room: &net_id::RoomName,
    me: &net_id::PeerId,
    presence: discovery::Presence,
    capacity: u16,
) -> Result<(), discovery::Error> {
    let members = BTreeMap::from([(me.clone(), presence)]);
    let update =
        discovery::Lobby::from([(room.clone(), discovery::RoomRecord { capacity, members })]);
    let request = ContractRequest::Update {
        key: lobby_client.key,
        data: UpdateData::Delta(StateDelta::from(bincode::serialize(&update)?)),
    };
    lobby_client
        .client
        .send(&ClientRequest::ContractOp(request))?;
    let cached = std::mem::take(&mut lobby_client.lobby);
    lobby_client.lobby = discovery::freenet::merge_lobby(cached, update);
    Ok(())
}

// no test_usage necessary
