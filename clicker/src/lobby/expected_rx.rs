use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Debug, Deref)]
pub struct ExpectedRx(pub Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<Vec<String>>>>);
