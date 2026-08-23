use bevy::prelude::Message;
use derive_more::Deref;

use crate::net_id;

#[derive(Message, Clone, Copy, PartialEq, Eq, Deref, Debug)]
pub struct Connected(pub net_id::NetworkId);
