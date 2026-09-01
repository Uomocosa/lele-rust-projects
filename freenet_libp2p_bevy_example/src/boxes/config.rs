use bevy::prelude::Resource;
use derive_more::Deref;

use super::config_new;
use crate::boxes;

#[derive(Resource, Debug, Clone, Copy, Deref)]
pub struct Config(pub boxes::PlayerId);

#[rustfmt::skip]
impl Config {
    pub fn new(own_id: boxes::PlayerId) -> Self { config_new::new(own_id) }
}
// no test_usage necessary — thin delegate, exercised by plugin.rs test_usage
