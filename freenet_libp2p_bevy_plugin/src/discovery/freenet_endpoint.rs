use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct FreenetEndpoint(pub u16);

#[cfg(test)]
mod tests {
    use super::FreenetEndpoint;

    #[test]
    fn test_usage() {
        let endpoint = FreenetEndpoint(7509);
        assert_eq!(*endpoint, 7509);
    }
}
