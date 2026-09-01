use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct Tick(pub u64);

#[cfg(test)]
mod tests {
    use super::Tick;

    #[test]
    fn test_usage() {
        assert_eq!(Tick(7).0, 7);
    }
}
