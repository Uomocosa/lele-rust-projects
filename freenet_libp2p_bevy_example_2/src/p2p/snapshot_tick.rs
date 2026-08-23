use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Deref, DerefMut, Default)]
pub struct SnapshotTick(pub u64);
// no test_usage necessary — thin resource, exercised by send_snapshot tests
