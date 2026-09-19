use std::time::Duration;

use super::chaos::Chaos;
use super::ghost_hint::GhostHint;
use super::nat::Nat;
use super::topology::Topology;

pub struct NetworkProfile {
    pub name: &'static str,
    pub seed: u64,
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub fail_rate: f64,
    pub tcp_capacity: usize,
    pub dial_within: Duration,
    pub redial_every: Duration,
    pub stagger: Duration,
    pub link_down_after: Duration,
    pub slow_pairs: Vec<(&'static str, &'static str, Duration, f64)>,
    pub topology: Topology,
    pub nat: Nat,
    pub ghosts: &'static [GhostHint],
    pub chaos: Chaos,
}
