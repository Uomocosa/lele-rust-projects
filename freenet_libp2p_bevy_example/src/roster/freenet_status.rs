use bevy::prelude::Resource;

#[derive(Debug, Clone, PartialEq, Eq, Resource, Default)]
pub enum FreenetStatus {
    #[default]
    Connecting,
    Connected,
    Retrying {
        attempt: u32,
    },
}
