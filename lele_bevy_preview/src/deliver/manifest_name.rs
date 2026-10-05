use crate::deliver::basic::constants;

#[must_use]
pub fn manifest_name() -> String {
    String::from(constants::MANIFEST_FILE)
}

// no test_usage necessary
