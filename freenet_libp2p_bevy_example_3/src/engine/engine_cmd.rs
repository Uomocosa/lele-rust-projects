use crate::engine;

/// Commands sent from the render `App` to the deterministic sim worker. `Step` advances the plain
/// authoritative engine (the source of truth for remote rendering and the convergence hash);
/// `Predict` advances the rollback session's predicted state for the local box; `Spawn` registers a
/// participant in both.
#[derive(Debug)]
pub enum EngineCmd {
    Spawn(engine::PlayerId),
    Step {
        tick: u64,
        actions: Vec<(engine::PlayerId, engine::Action)>,
    },
    Predict {
        inputs: Vec<(engine::PlayerId, engine::Action)>,
    },
}

#[cfg(test)]
mod tests {
    use crate::engine;

    use super::EngineCmd;

    #[test]
    fn test_usage() {
        let command = EngineCmd::Spawn([1; 32]);
        assert!(matches!(command, EngineCmd::Spawn(_)));
        let action = engine::Action {
            direction: engine::Direction::Right,
            jump: false,
        };
        let _ = EngineCmd::Step {
            tick: 1,
            actions: vec![([1; 32], action)],
        };
        let _ = EngineCmd::Predict {
            inputs: vec![([1; 32], action)],
        };
    }
}
