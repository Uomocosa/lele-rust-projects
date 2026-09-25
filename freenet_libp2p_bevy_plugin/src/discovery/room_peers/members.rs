use std::collections::BTreeMap;

use super::member::Member;
use crate::discovery;

pub type Members = BTreeMap<discovery::id::RemotePeerId, Member>;
// no test_usage necessary
