// VIOLATION: thin delegate NOT annotated with #[rustfmt::skip]
use super::counter_increment;

pub struct Counter {
    pub count: u32,
}

impl Counter {
    pub fn increment(&mut self) { counter_increment::increment(self) }

    // VIOLATION: real-body method with >3 statements, must be extracted to counter_double.rs
    pub fn double(&mut self) {
        let previous = self.count;
        let doubled = previous * 2;
        self.count = doubled;
        self.count += 0;
    }
}

// VIOLATION: trait impl with >3-statement real body, must be extracted to method files
use std::fmt;
impl fmt::Display for Counter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = "Counter";
        let value = self.count;
        let rendered = format!("{label}({value})");
        write!(f, "{rendered}")
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
