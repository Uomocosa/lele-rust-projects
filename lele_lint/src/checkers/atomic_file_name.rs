use super::atomic_file::AtomicFile;

pub(crate) fn name(_self: &AtomicFile) -> &'static str {
    "atomic_file"
}

// no test_usage necessary
