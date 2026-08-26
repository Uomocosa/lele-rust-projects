#[derive(Clone, Debug)]
pub struct TickSample {
    pub secs: u64,
    pub count: u64,
    pub owns: u64,
}

// no test_usage necessary — pure data type
