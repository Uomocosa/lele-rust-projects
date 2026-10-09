use bevy::prelude::Reflect;
use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, From)]
#[from(forward)]
pub struct GameName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, From)]
#[from(forward)]
pub struct GameToken(pub String);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref, Reflect,
)]
pub struct EpochSecs(pub u64);
