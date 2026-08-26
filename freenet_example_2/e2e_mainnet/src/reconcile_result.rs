pub struct ReconcileResult {
    pub final_counts: Vec<u64>,
    pub reconciled: bool,
    pub latency_secs: Option<u64>,
    pub expected_union: Option<u64>,
    pub merged_correct: Option<bool>,
    pub aggregated: bool,
}

// no test_usage necessary — pure data type
