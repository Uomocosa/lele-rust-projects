// VIOLATION: thin delegate NOT annotated with #[rustfmt::skip]
use super::counter_increment;

pub struct Counter {
    pub count: u32,
}

impl Counter {
    pub fn increment(&mut self) { counter_increment::increment(self) }
    // VIOLATION: import uses super:: for method file (good) but #[rustfmt::skip] is missing
    // VIOLATION: dispatch not 2 segments: counter_increment::increment is correct (2 segments),
    //            but having multiple methods in one delegate block is fine
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
