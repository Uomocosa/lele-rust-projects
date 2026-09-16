use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub struct DirectoryLive(pub bool);
