use super::instance_outcome::InstanceOutcome;

pub struct Outcome {
    pub instances: Vec<InstanceOutcome>,
    pub put_count: usize,
    pub error_sigs: Vec<String>,
    pub max_offline_secs: f64,
}

#[rustfmt::skip]
impl Outcome {
    pub fn all_moved(&self) -> bool { self.instances.iter().all(|i| i.moved) }
    pub fn all_converged(&self) -> bool {
        let expected = self.instances.len().saturating_sub(1);
        self.instances.iter().all(|i| i.applied_peer_ids >= expected)
    }
    pub fn within_flicker_budget(&self, allowed: f64) -> bool { self.max_offline_secs <= allowed }
}

// no test_usage necessary
