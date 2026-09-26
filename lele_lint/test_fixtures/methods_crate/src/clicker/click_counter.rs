use derive_more::{Deref, DerefMut};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct ClickCounter(pub i32);

#[atomic_delegates]
impl ClickCounter {
    pub fn increment(&mut self) {}
    pub fn add(&mut self, delta: i32) {}
}

#[cfg(test)]
mod tests {
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut counter = clicker::ClickCounter::default();
        counter.increment();
        assert_eq!(*counter, 1);
    }
}
