use bevy::prelude::Resource;
use derive_more::Deref;

use super::config_new;
use freenet_libp2p_bevy_plugin::net_id;

#[derive(Resource, Debug, Clone, Copy, Deref)]
pub struct Config(pub net_id::NetworkId);

#[rustfmt::skip]
impl Config {
    #[must_use]
    pub const fn new(own_id: net_id::NetworkId) -> Self { config_new::new(own_id) }
}
// no test_usage necessary — thin delegate, exercised by plugin tests
