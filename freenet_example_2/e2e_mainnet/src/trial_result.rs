pub struct TrialResult {
    pub mode: String,
    pub rep: usize,
    pub ready: bool,
    pub put_count: usize,
    pub error_sigs: Vec<String>,
    pub final_counts: Vec<u64>,
    pub reconciled: bool,
    pub latency_secs: Option<u64>,
    pub expected_union: Option<u64>,
    pub merged_correct: Option<bool>,
    pub aggregated: bool,
    pub bridge_splits: usize,
    pub bridge_merges: usize,
    pub video: Option<Vec<u8>>,
    pub run_label: String,
}

// no test_usage necessary — pure data type
