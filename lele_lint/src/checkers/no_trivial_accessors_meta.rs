// no test_usage necessary
use super::no_trivial_accessors::NoTrivialAccessors;

pub(crate) fn name(_self: &NoTrivialAccessors) -> &'static str {
    "no_trivial_accessors"
}

pub(crate) fn code(_self: &NoTrivialAccessors) -> &'static str {
    "E010"
}
