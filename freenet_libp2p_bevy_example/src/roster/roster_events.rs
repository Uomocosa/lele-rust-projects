use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::roster;

#[derive(Resource, Deref, DerefMut)]
pub struct RosterEvents(pub tokio::sync::mpsc::UnboundedReceiver<roster::Event>);
