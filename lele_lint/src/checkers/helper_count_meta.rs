// no test_usage necessary
use super::helper_count::HelperCount;

pub(crate) fn name(_self: &HelperCount) -> &'static str {
    "helper_count"
}

pub(crate) fn code(_self: &HelperCount) -> &'static str {
    "E015"
}
