use crate::engine;
use crate::netcode;

pub fn simulate_lockstep(
    participants: &[engine::PlayerId],
    inputs: &[(u64, Vec<(engine::PlayerId, engine::Action)>)],
) -> Vec<u64> {
    let mut lockstep = netcode::Lockstep::new(participants.to_vec());
    let mut engine = engine::Engine::new();
    for participant in participants {
        engine.spawn_player(*participant);
    }

    let mut hashes = Vec::new();
    for (now, (_, tick_inputs)) in inputs.iter().enumerate() {
        let now = now as u64;
        for (peer, action) in tick_inputs {
            lockstep
                .record_commit(now, *peer, engine::hash_action(action))
                .unwrap();
            lockstep.record_reveal(now, *peer, *action).unwrap();
        }
        apply_plans(&mut lockstep, &mut engine, &mut hashes, now);
    }

    let drain_now = inputs.len() as u64 + netcode::constants::COMMAND_DELAY;
    apply_plans(&mut lockstep, &mut engine, &mut hashes, drain_now);
    hashes
}

// needed helper:
fn apply_plans(
    lockstep: &mut netcode::Lockstep,
    engine: &mut engine::Engine,
    hashes: &mut Vec<u64>,
    now_tick: u64,
) {
    for plan in lockstep.advance_to(now_tick) {
        let snapshot = engine.step(plan.tick, &plan.ordered_inputs).unwrap();
        hashes.push(engine::hash_snapshot(&snapshot));
    }
}

#[cfg(test)]
mod tests {
    use crate::engine;

    use super::simulate_lockstep;

    fn honest_inputs(tick: u64) -> Vec<(engine::PlayerId, engine::Action)> {
        let jump = tick.is_multiple_of(20);
        let direction = if tick.is_multiple_of(2) {
            engine::Direction::Right
        } else {
            engine::Direction::Left
        };
        vec![
            ([1; 32], engine::Action { direction, jump }),
            (
                [2; 32],
                engine::Action {
                    direction: engine::Direction::Center,
                    jump: false,
                },
            ),
        ]
    }

    #[test]
    fn test_usage() {
        let participants = [[1; 32], [2; 32]];
        let trace = (0..10)
            .map(|tick| (tick, honest_inputs(tick)))
            .collect::<Vec<_>>();
        let hashes = simulate_lockstep(&participants, &trace);
        assert_eq!(hashes.len(), trace.len());
        assert!(hashes.windows(2).any(|w| w[0] != w[1]));
    }

    #[test]
    fn two_nodes_converge_on_same_state_hash() {
        let participants = [[1; 32], [2; 32]];
        let trace = (0..90)
            .map(|tick| (tick, honest_inputs(tick)))
            .collect::<Vec<_>>();

        let node_a = simulate_lockstep(&participants, &trace);
        let node_b = simulate_lockstep(&participants, &trace);

        assert_eq!(node_a.len(), node_b.len());
        for (index, (a, b)) in node_a.iter().zip(node_b.iter()).enumerate() {
            assert_eq!(
                a, b,
                "two independent peers diverged at applied tick index {index}"
            );
        }
    }
}
