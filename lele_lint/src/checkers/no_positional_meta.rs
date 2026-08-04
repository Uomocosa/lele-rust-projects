// no test_usage necessary
use super::no_positional::NoPositional;

pub(crate) fn name(_self: &NoPositional) -> &'static str {
    "no_positional"
}

pub(crate) fn code(_self: &NoPositional) -> &'static str {
    "E009"
}
