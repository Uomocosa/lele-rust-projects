// no test_usage necessary
use super::test_inline::TestInline;

pub(crate) fn name(_self: &TestInline) -> &'static str {
    "test_inline"
}

pub(crate) fn code(_self: &TestInline) -> &'static str {
    "E007"
}
