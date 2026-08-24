use crate::netcode;

/// Advances the applied window up to `now - D`. Ticks are applied once the fixed command delay `D`
/// has elapsed; a participant missing a reveal for the tick is treated as null and, once its late
/// streak exceeds the liveness budget `B`, is marked offline and excluded.
pub fn advance_to(lockstep: &mut netcode::Lockstep, now_tick: u64) -> Vec<netcode::TickPlan> {
    let apply_window = now_tick.saturating_sub(netcode::constants::COMMAND_DELAY);
    let mut applied = Vec::new();
    while lockstep.applied_through < apply_window {
        let tick = lockstep.applied_through + 1;
        apply_tick(lockstep, tick);
        applied.push(plan_for(lockstep, tick));
    }
    applied
}

fn plan_for(lockstep: &netcode::Lockstep, tick: u64) -> netcode::TickPlan {
    let mut ordered = Vec::new();
    let mut late = Vec::new();
    for peer in &lockstep.participants {
        if lockstep.offline.contains(peer) {
            continue;
        }
        let action = lockstep
            .reveals
            .get(&(tick, *peer))
            .copied()
            .unwrap_or_default();
        ordered.push((*peer, action));
        if !lockstep.reveals.contains_key(&(tick, *peer)) {
            late.push(*peer);
        }
    }
    ordered.sort_unstable_by_key(|(id, _)| *id);
    netcode::TickPlan {
        tick,
        ordered_inputs: ordered,
        late,
        offline: lockstep.offline.clone(),
        tampered: lockstep.tampered.iter().copied().collect(),
    }
}

// needed helper:
fn apply_tick(lockstep: &mut netcode::Lockstep, tick: u64) {
    let participants = lockstep.participants.clone();
    for peer in &participants {
        if lockstep.offline.contains(peer) {
            continue;
        }
        if lockstep.reveals.contains_key(&(tick, *peer)) {
            let count = lockstep.late_streak.entry(*peer).or_insert(0);
            *count = 0;
        } else {
            let count = lockstep.late_streak.entry(*peer).or_insert(0);
            *count += 1;
            if *count >= netcode::constants::LIVENESS_BUDGET && !lockstep.offline.contains(peer) {
                lockstep.offline.push(*peer);
            }
        }
    }
    lockstep.applied_through = tick;
}

#[cfg(test)]
mod tests {
    use crate::engine;
    use crate::netcode;

    #[test]
    fn test_usage() {
        let mut lockstep = netcode::Lockstep::new(vec![[1; 32]]);
        let action = engine::Action {
            direction: engine::Direction::Right,
            jump: false,
        };
        lockstep
            .record_commit(1, [1; 32], engine::hash_action(&action))
            .unwrap();
        lockstep.record_reveal(1, [1; 32], action).unwrap();

        lockstep.advance_to(0);
        lockstep.advance_to(netcode::constants::COMMAND_DELAY);
        let plans = lockstep.advance_to(netcode::constants::COMMAND_DELAY + 1);
        let plan = plans.last().unwrap();
        assert_eq!(plan.tick, 1);
        assert_eq!(plan.ordered_inputs, vec![([1; 32], action)]);
        assert!(plan.late.is_empty());
    }

    #[test]
    fn slow_peer_is_null_then_offline_never_favored() {
        let fast = [1; 32];
        let slow = [2; 32];
        let mut lockstep = netcode::Lockstep::new(vec![slow, fast]);
        let d = netcode::constants::COMMAND_DELAY;

        let run_action = engine::Action {
            direction: engine::Direction::Right,
            jump: false,
        };
        for tick in 1..=300 {
            let now = tick + d;
            lockstep
                .record_commit(tick, fast, engine::hash_action(&run_action))
                .unwrap();
            lockstep
                .record_commit(tick, slow, engine::hash_action(&run_action))
                .unwrap();
            assert_eq!(lockstep.record_reveal(tick, fast, run_action), Ok(()));
            let plans = lockstep.advance_to(now);
            if let Some(plan) = plans.last() {
                assert!(plan.late.contains(&slow) || lockstep.offline.contains(&slow));
                let fast_used = plan
                    .ordered_inputs
                    .iter()
                    .any(|(id, action)| *id == fast && !action.is_null());
                let slow_used = plan
                    .ordered_inputs
                    .iter()
                    .any(|(id, action)| *id == slow && !action.is_null());
                assert!(
                    fast_used,
                    "fast peer's insight is used once all commits are in"
                );
                assert!(!slow_used, "slow peer's input is never used");
            }
        }

        assert!(
            lockstep.offline.contains(&slow),
            "slow peer is excluded once past the liveness budget"
        );
        assert!(!lockstep.offline.contains(&fast));
        assert_eq!(
            netcode::constants::COMMAND_DELAY,
            d,
            "the command delay is a fixed constant, never extended for a slow peer"
        );
    }
}
