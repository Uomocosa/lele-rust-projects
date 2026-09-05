use bevy::prelude::Component;
use derive_more::{Deref, DerefMut};

use super::click_counter_decrement;
use super::click_counter_increment;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct ClickCounter(pub i32);

#[rustfmt::skip]
impl ClickCounter {
    pub fn increment(&mut self) { click_counter_increment::increment(self) }
    pub fn decrement(&mut self) { click_counter_decrement::decrement(self) }
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
