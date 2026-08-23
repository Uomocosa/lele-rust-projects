use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::p2p;

#[derive(Resource, Deref, DerefMut)]
pub struct P2pEvents<T: p2p::Message>(pub tokio::sync::mpsc::UnboundedReceiver<p2p::Event<T>>);
// no test_usage necessary — thin resource, exercised by bevy_systems tests
