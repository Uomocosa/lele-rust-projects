use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::engine;

/// Predicted (local-ahead) snapshot produced by the rollback session; used to render the local
/// box immediately while the authoritative snapshot (`LatestSnapshot`) drives remote boxes.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct PredictedSnapshot(pub Option<engine::Snapshot>);
// no test_usage necessary - thin resource, filled by the boxes netcode_tick system
