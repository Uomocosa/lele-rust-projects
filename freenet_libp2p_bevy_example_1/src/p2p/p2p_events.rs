use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::p2p;

#[derive(Resource, Deref, DerefMut)]
pub struct P2pEvents(pub tokio::sync::mpsc::UnboundedReceiver<p2p::Event>);
// no test_usage necessary — thin resource, exercised by bevy_systems tests
