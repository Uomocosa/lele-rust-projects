use std::collections::HashSet;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Deref, DerefMut, Default)]
pub struct DialedPeers(pub HashSet<String>);
// no test_usage necessary — thin resource, exercised by dial_roster_peers tests
