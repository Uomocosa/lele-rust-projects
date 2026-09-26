use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct GlobalCounter(pub i32);

#[atomic_delegates]
impl GlobalCounter {
    pub fn increment(&mut self) {}
    pub fn add(&mut self, delta: i32) {}
}

#[cfg(test)]
mod tests {
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut counter = clicker::GlobalCounter::default();
        counter.increment();
        assert_eq!(*counter, 1);
    }
}
