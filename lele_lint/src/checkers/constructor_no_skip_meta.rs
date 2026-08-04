// no test_usage necessary
use super::constructor_no_skip::ConstructorNoSkip;

pub(crate) fn name(_self: &ConstructorNoSkip) -> &'static str {
    "constructor_no_skip"
}

pub(crate) fn code(_self: &ConstructorNoSkip) -> &'static str {
    "E013"
}
