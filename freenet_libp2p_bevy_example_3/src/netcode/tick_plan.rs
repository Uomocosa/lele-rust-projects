use crate::engine;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TickPlan {
    pub tick: u64,
    pub ordered_inputs: Vec<(engine::PlayerId, engine::Action)>,
    pub late: Vec<engine::PlayerId>,
    pub offline: Vec<engine::PlayerId>,
    pub tampered: Vec<engine::PlayerId>,
}

#[cfg(test)]
mod tests {
    use super::TickPlan;

    #[test]
    fn test_usage() {
        let plan = TickPlan::default();
        assert_eq!(plan.tick, 0);
        assert!(plan.ordered_inputs.is_empty());
    }
}
