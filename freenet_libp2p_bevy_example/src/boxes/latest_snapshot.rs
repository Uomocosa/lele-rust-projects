use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::engine;

#[derive(Resource, Deref, DerefMut, Default)]
pub struct LatestSnapshot(pub Option<engine::Snapshot>);
// no test_usage necessary - thin resource, filled by the boxes netcode_tick system
