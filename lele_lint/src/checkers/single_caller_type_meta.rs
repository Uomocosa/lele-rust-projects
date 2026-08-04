// no test_usage necessary
use super::single_caller_type::SingleCallerType;

pub(crate) fn name(_self: &SingleCallerType) -> &'static str {
    "single_caller_type"
}

pub(crate) fn code(_self: &SingleCallerType) -> &'static str {
    "E016"
}
