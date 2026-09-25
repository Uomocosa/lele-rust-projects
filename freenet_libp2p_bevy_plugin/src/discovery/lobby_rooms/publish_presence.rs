use std::collections::BTreeMap;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::{StateDelta, UpdateData};

use super::super::error::Error;
use super::super::id::EpochSecs;
use super::super::id::Presence;
use super::super::id::RemotePeerId;
use super::super::id::RoomName;
use super::super::id::RoomRecord;
use super::IndexClient;
use super::RoomCatalogue;
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
