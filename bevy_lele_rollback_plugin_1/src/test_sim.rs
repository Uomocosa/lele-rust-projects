use super::simulation::Simulation;

#[derive(Default)]
pub struct TestSim {
    position: i64,
    velocity: i64,
}

#[rustfmt::skip]
impl Simulation for TestSim {
    type State = (i64, i64);
    type Input = i64;

    fn step(&mut self, _tick: u64, inputs: &[i64]) {
        for &delta in inputs {
            self.velocity += delta;
            self.position += self.velocity;
        }
    }

    fn snapshot(&self) -> (i64, i64) {
        (self.position, self.velocity)
    }

    fn restore(&mut self, state: (i64, i64)) {
        let (position, velocity) = state;
        self.position = position;
        self.velocity = velocity;
    }

    fn hash(&self) -> u64 {
        ((self.position as u64) << 32) ^ (self.velocity as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::Simulation;
    use super::TestSim;

    #[test]
    fn test_usage() {
        let mut sim = TestSim::default();
        sim.step(1, &[3]);
        let (position, velocity) = sim.snapshot();
        assert_eq!((position, velocity), (3, 3));
        let hash = sim.hash();
        sim.restore(sim.snapshot());
        assert_eq!(sim.hash(), hash);
    }
}
