use derive_more::Deref;
use serde::{Deserialize, Serialize};

use crate::frame::Frame;

#[derive(Clone, Debug, Serialize, Deserialize, Deref)]
pub struct LetterRequest(pub Frame);

// no test_usage necessary
