use bevy::prelude::Resource;
use derive_more::Deref;

use crate::p2p;

#[derive(Resource, Deref)]
pub struct P2pCommands<T: p2p::Message>(pub tokio::sync::mpsc::UnboundedSender<p2p::Command<T>>);
// no test_usage necessary — thin resource, exercised by bevy_systems tests
