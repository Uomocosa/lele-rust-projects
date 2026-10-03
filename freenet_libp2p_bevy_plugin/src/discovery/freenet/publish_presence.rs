use std::collections::BTreeMap;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::{StateDelta, UpdateData};

use crate::discovery;

pub fn publish_presence(
    directory_client: &mut discovery::freenet::DirectoryClient,
    room: &discovery::RoomName,
    me: &discovery::PeerId,
    presence: discovery::Presence,
    capacity: u16,
) -> Result<(), discovery::Error> {
    let members = BTreeMap::from([(me.clone(), presence)]);
    let update =
        discovery::Directory::from([(room.clone(), discovery::RoomRecord { capacity, members })]);
    let request = ContractRequest::Update {
        key: directory_client.key,
        data: UpdateData::Delta(StateDelta::from(bincode::serialize(&update)?)),
    };
    directory_client
        .client
        .send(&ClientRequest::ContractOp(request))?;
    let cached = std::mem::take(&mut directory_client.directory);
    directory_client.directory = discovery::freenet::merge_directory(cached, update);
    Ok(())
}

// no test_usage necessary
