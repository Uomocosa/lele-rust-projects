use derive_more::Deref;
use serde::{Deserialize, Serialize};

use crate::frame;

#[derive(Clone, Debug, Serialize, Deserialize, Deref)]
pub struct LetterRequest(pub frame::Frame);

// no test_usage necessary
