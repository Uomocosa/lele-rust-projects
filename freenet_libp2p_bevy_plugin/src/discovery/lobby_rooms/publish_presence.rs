use std::collections::BTreeMap;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::{StateDelta, UpdateData};

use super::super::error::Error;
use super::super::id::epoch_secs::EpochSecs;
use super::super::id::presence::Presence;
use super::super::id::remote_peer_id::RemotePeerId;
use super::super::id::room_name::RoomName;
use super::super::id::room_record::RoomRecord;
use super::catalogue::RoomCatalogue;
use super::index_client::IndexClient;
use super::merge_board::merge_board;

pub fn publish_presence(
    client: &mut IndexClient,
    room: &RoomName,
    me: &RemotePeerId,
    addrs: &[String],
    updated_at: EpochSecs,
    capacity: u16,
) -> Result<(), Error> {
    let mut members: BTreeMap<RemotePeerId, Presence> = BTreeMap::new();
    members.insert(
        me.clone(),
        Presence {
            addrs: addrs.to_vec(),
            updated_at,
        },
    );
    let mut update: RoomCatalogue = BTreeMap::new();
    update.insert(room.clone(), RoomRecord { capacity, members });
    let data = bincode::serialize(&update)?;
    let request = ContractRequest::Update {
        key: client.contract_key,
        data: UpdateData::Delta(StateDelta::from(data)),
    };
    client.client.send(&ClientRequest::ContractOp(request))?;
    client.slots = merge_board(client.slots.clone(), update);
    Ok(())
}

// no test_usage necessary
