// no test_usage necessary
use super::test_usage::TestUsage;

pub(crate) fn name(_self: &TestUsage) -> &'static str {
    "test_usage"
}

pub(crate) fn code(_self: &TestUsage) -> &'static str {
    "E006"
}
