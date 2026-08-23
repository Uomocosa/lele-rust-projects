use bevy::prelude::Resource;
use derive_more::Deref;

use crate::net_id;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct LocalIdentity(pub net_id::NetworkId);
