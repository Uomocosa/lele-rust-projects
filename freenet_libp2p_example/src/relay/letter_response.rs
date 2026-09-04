use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Deref)]
pub struct LetterResponse(pub bool);

// no test_usage necessary
