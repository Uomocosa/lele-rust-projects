// no test_usage necessary
use super::method_visibility::MethodVisibility;

pub(crate) fn name(_self: &MethodVisibility) -> &'static str {
    "method_visibility"
}

pub(crate) fn code(_self: &MethodVisibility) -> &'static str {
    "E003"
}
