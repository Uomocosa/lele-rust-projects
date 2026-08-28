use crate::instance_outcome;

pub struct Outcome {
    pub instances: Vec<instance_outcome::InstanceOutcome>,
    pub put_count: usize,
    pub converged: bool,
    pub aggregated: bool,
    pub error_sigs: Vec<String>,
    pub bridge_splits: usize,
    pub bridge_merges: usize,
}
