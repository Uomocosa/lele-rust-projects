use std::time::Duration;

use super::ghost_hint::GhostHint;
use super::nat::Nat;

#[derive(Clone)]
pub struct DialPlan {
    pub name: &'static str,
    pub own: u64,
    pub clicks: u32,
    pub dial: Vec<&'static str>,
    pub accept: usize,
    pub ghosts: &'static [GhostHint],
    pub public_ip: &'static str,
    pub nat: Nat,
    pub link_down_after: Duration,
    pub dial_within: Duration,
    pub redial_every: Duration,
    pub listen_at: Duration,
}
