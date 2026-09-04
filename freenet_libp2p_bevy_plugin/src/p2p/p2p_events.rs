use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::p2p;

#[derive(Resource, Deref, DerefMut)]
pub struct P2PEvents<T: p2p::Message>(pub Vec<p2p::Event<T>>);

impl<T: p2p::Message> Default for P2PEvents<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::P2PEvents;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let e = P2PEvents::<Dummy>::default();
        assert!(e.is_empty());
    }
}
