use std::collections::BTreeMap;

use super::entry::Entry;

pub type DirectoryState = BTreeMap<String, Entry>;
// no test_usage necessary
