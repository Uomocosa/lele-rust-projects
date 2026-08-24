use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::lockstep_advance_to;
use super::lockstep_new;
use super::lockstep_record_commit;
use super::lockstep_record_reveal;
use super::lockstep_sync_participants;
use crate::engine;
use crate::netcode;

#[derive(Debug, Clone)]
pub struct Lockstep {
    pub participants: Vec<engine::PlayerId>,
    pub commits: BTreeMap<(u64, engine::PlayerId), u64>,
    pub reveals: BTreeMap<(u64, engine::PlayerId), engine::Action>,
    pub tampered: BTreeSet<engine::PlayerId>,
    pub late_streak: BTreeMap<engine::PlayerId, u64>,
    pub offline: Vec<engine::PlayerId>,
    pub applied_through: u64,
    pub tick_plans: VecDeque<netcode::TickPlan>,
}

impl Default for Lockstep {
    fn default() -> Self {
        lockstep_new::new(Vec::new())
    }
}

#[rustfmt::skip]
impl Lockstep {
    pub fn new(participants: Vec<engine::PlayerId>) -> Self { lockstep_new::new(participants) }
    pub fn record_commit(&mut self, tick: u64, peer: engine::PlayerId, hash: u64) -> Result<(), netcode::Error> { lockstep_record_commit::record_commit(self, tick, peer, hash) }
    pub fn record_reveal(&mut self, tick: u64, peer: engine::PlayerId, action: engine::Action) -> Result<(), netcode::Error> { lockstep_record_reveal::record_reveal(self, tick, peer, action) }
    pub fn advance_to(&mut self, now_tick: u64) -> Vec<netcode::TickPlan> { lockstep_advance_to::advance_to(self, now_tick) }
    pub fn sync_participants(&mut self, ids: &[engine::PlayerId]) { lockstep_sync_participants::sync_participants(self, ids) }
}
// no test_usage necessary — thin delegates, covered by lockstep tests
