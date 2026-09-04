use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::p2p;

#[derive(Resource, Deref, DerefMut)]
pub struct P2PCommands<T: p2p::Message>(pub Vec<p2p::Command<T>>);

impl<T: p2p::Message> Default for P2PCommands<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::P2PCommands;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let c = P2PCommands::<Dummy>::default();
        assert!(c.is_empty());
    }
}
