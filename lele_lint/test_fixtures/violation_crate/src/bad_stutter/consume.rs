use crate::report;

pub fn consume(r: report::Report) -> String {
    r.title.clone()
}

// no test_usage necessary
