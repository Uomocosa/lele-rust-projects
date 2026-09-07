use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use super::global_counter_add;
use super::global_counter_increment;

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct GlobalCounter(pub i32);

#[rustfmt::skip]
impl GlobalCounter {
    pub fn increment(&mut self) { global_counter_increment::increment(self) }
    pub fn add(&mut self, delta: i32) { global_counter_add::add(self, delta) }
}

#[cfg(test)]
mod tests {
    use super::GlobalCounter;

    #[test]
    fn test_usage() {
        let mut counter = GlobalCounter::default();
        counter.increment();
        counter.increment();
        assert_eq!(*counter, 2);
        counter.add(40);
        assert_eq!(*counter, 42);
    }
}
