// no test_usage necessary
use super::atomic_file::AtomicFile;

pub(crate) fn name(_self: &AtomicFile) -> &'static str {
    "atomic_file"
}

pub(crate) fn code(_self: &AtomicFile) -> &'static str {
    "E001"
}
