use std::collections::BTreeMap;

use crate::engine;

/// Canonical, fully restorable snapshot of the deterministic sim used as the rollback
/// `Simulation::State`. Carries velocity as well as position so an avian rollback (restore +
/// re-step) reproduces the exact authoritative trajectory.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EngineSimState {
    pub tick: u64,
    pub bodies: BTreeMap<engine::PlayerId, (f32, f32, f32, f32)>,
}

#[cfg(test)]
mod tests {
    use super::EngineSimState;

    #[test]
    fn test_usage() {
        let mut bodies = std::collections::BTreeMap::new();
        bodies.insert([1; 32], (1.0, 2.0, 3.0, 0.0));
        let state = EngineSimState { tick: 4, bodies };
        assert_eq!(state.tick, 4);
        assert_eq!(state.bodies[&[1; 32]].0, 1.0);
    }
}
