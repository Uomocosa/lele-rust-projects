// VIOLATION: thin delegate NOT annotated with #[rustfmt::skip]
use super::counter_increment;

pub struct Counter {
    pub count: u32,
}

impl Counter {
    pub fn increment(&mut self) { counter_increment::increment(self) }

    // VIOLATION: real-body method, must be extracted to counter_double.rs
    pub fn double(&mut self) {
        self.count *= 2;
    }
}

// VIOLATION: trait impl with real body, must be extracted to method files
use std::fmt;
impl fmt::Display for Counter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Counter({})", self.count)
    }
}

#[cfg(test)]
mod tests {
    use super::Counter;

    #[test]
    fn test_usage() {
        let mut c = Counter { count: 0 };
        c.increment();
        assert_eq!(c.count, 1);
    }
}
