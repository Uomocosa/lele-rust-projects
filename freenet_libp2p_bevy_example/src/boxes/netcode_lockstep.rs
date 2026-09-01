use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::netcode;

#[derive(Resource, Deref, DerefMut, Default)]
pub struct NetcodeLockstep(pub netcode::Lockstep);
// no test_usage necessary - thin resource, exercised by the boxes integration tests
