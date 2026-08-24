pub trait Simulation {
    type State;
    type Input;

    fn step(&mut self, tick: u64, inputs: &[Self::Input]);

    fn snapshot(&self) -> Self::State;

    fn restore(&mut self, state: Self::State);

    fn hash(&self) -> u64;
}

#[cfg(test)]
mod tests {
    use super::super::test_sim;
    use super::Simulation;

    #[test]
    fn test_usage() {
        let mut sim = test_sim::TestSim::default();
        sim.step(1, &[2, 3]);
        let before = sim.hash();
        assert_ne!(before, test_sim::TestSim::default().hash());
        let state = sim.snapshot();
        sim.restore(state);
        assert_eq!(sim.hash(), before);
    }
}
