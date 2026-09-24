use std::collections::BTreeMap;

use super::directory_entry::DirectoryEntry;

pub type DirectoryState = BTreeMap<String, DirectoryEntry>;
// no test_usage necessary
