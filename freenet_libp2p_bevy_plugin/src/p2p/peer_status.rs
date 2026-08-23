use std::collections::HashSet;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Deref, DerefMut, Default)]
pub struct PeerStatus(pub HashSet<String>);
// no test_usage necessary — thin resource, exercised by poll_swarm_events tests
