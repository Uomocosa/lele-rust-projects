use bevy::prelude::Component;
use derive_more::{Deref, DerefMut};

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct ClickCounter(pub i32);

#[rustfmt::skip]
#[allow(clippy::arithmetic_side_effects)]
impl ClickCounter {
    pub fn increment(&mut self) { **self += 1; }
    pub fn decrement(&mut self) { **self -= 1; }
}

#[cfg(test)]
mod tests {
    use super::ClickCounter;

    #[test]
    fn test_usage() {
        let mut counter = ClickCounter::default();
        counter.increment();
        counter.increment();
        assert_eq!(*counter, 2);
        counter.decrement();
        assert_eq!(*counter, 1);
        assert_eq!(*counter, 1);
    }
}
