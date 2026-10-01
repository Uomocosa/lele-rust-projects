use bevy::prelude::Reflect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum DiscoveryStatus {
    #[default]
    Known,
    Connected,
}
