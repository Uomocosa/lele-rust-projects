use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct DirectoryLive(pub bool);

#[cfg(test)]
mod tests {
    use super::DirectoryLive;

    #[test]
    fn test_usage() {
        let mut live = DirectoryLive::default();
        assert!(!live.0);
        live.0 = true;
        assert!(live.0);
    }
}
