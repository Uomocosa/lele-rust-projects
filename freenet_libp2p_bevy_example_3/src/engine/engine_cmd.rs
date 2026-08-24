use crate::engine;

#[derive(Debug)]
pub enum EngineCmd {
    Spawn(engine::PlayerId),
    Step {
        tick: u64,
        actions: Vec<(engine::PlayerId, engine::Action)>,
    },
}

#[cfg(test)]
mod tests {
    use crate::engine;

    use super::EngineCmd;

    #[test]
    fn test_usage() {
        let cmd = EngineCmd::Spawn([1; 32]);
        let _ = cmd;
        let _ = EngineCmd::Step {
            tick: 3,
            actions: vec![(
                [1; 32],
                engine::Action {
                    direction: engine::Direction::Right,
                    jump: false,
                },
            )],
        };
    }
}
