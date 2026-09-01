use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::boxes;
use crate::roster;

#[derive(Resource, Debug, Default, Clone, Deref, DerefMut)]
pub struct Roster(pub BTreeMap<boxes::PlayerId, roster::PeerEntry>);
