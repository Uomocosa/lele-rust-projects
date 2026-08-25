use crate::engine;

pub fn run_trace(trace: &[(u64, Vec<(engine::PlayerId, engine::Action)>)]) -> Vec<u64> {
    let mut engine = engine::Engine::new();
    for id in [[1; 32], [2; 32]] {
        engine.spawn_player(id);
    }
    let mut hashes = Vec::new();
    for (tick, actions) in trace {
        let snapshot = engine.step(*tick, actions).unwrap();
        hashes.push(engine::hash_snapshot(&snapshot));
    }
    hashes
}

#[cfg(test)]
mod tests {
    use crate::engine;

    use super::run_trace;

    fn tick_actions(
        tick: u64,
        a_jump: bool,
        b_dir: engine::Direction,
    ) -> (u64, Vec<(engine::PlayerId, engine::Action)>) {
        let actions = vec![
            (
                [1; 32],
                engine::Action {
                    direction: engine::Direction::Right,
                    jump: a_jump,
                },
            ),
            (
                [2; 32],
                engine::Action {
                    direction: b_dir,
                    jump: false,
                },
            ),
        ];
        (tick, actions)
    }

    fn gravity_trace() -> Vec<(u64, Vec<(engine::PlayerId, engine::Action)>)> {
        (0..90)
            .map(|tick| tick_actions(tick, false, engine::Direction::Center))
            .collect()
    }

    #[test]
    fn test_usage() {
        let trace = gravity_trace();
        let hashes = run_trace(&trace);
        assert_eq!(hashes.len(), trace.len());
        assert!(hashes.windows(2).any(|w| w[0] != w[1]));
    }

    #[test]
    fn same_trace_produces_identical_state_hash() {
        let trace = (0..120)
            .map(|tick| {
                let jump = tick == 20;
                let dir = if tick % 3 == 0 {
                    engine::Direction::Left
                } else {
                    engine::Direction::Right
                };
                tick_actions(
                    tick,
                    jump,
                    if tick % 2 == 0 {
                        dir
                    } else {
                        engine::Direction::Center
                    },
                )
            })
            .collect::<Vec<_>>();

        let first = run_trace(&trace);
        let second = run_trace(&trace);

        assert_eq!(first.len(), trace.len());
        assert_eq!(second.len(), trace.len());
        for (index, (a, b)) in first.iter().zip(second.iter()).enumerate() {
            assert_eq!(
                a, b,
                "state hash diverged at tick index {index} for identical traces"
            );
        }
        assert_eq!(first[first.len() - 1], second[second.len() - 1]);
    }

    #[test]
    fn identity_and_a_jump_move_differ() {
        let right_jump = (0..60)
            .map(|tick| tick_actions(tick, tick == 10, engine::Direction::Right))
            .collect::<Vec<_>>();
        let center = (0..60)
            .map(|tick| tick_actions(tick, true, engine::Direction::Center))
            .collect::<Vec<_>>();

        let a = run_trace(&right_jump);
        let b = run_trace(&center);
        assert_ne!(a[a.len() - 1], b[b.len() - 1]);
    }
}
