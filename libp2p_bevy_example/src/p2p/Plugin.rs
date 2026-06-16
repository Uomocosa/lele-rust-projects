use bevy::prelude::*;

use crate::p2p::Config;
use crate::p2p::PluginMethod;

pub struct Plugin {
    pub(crate) config: Config,
}

#[rustfmt::skip]
impl Plugin {
    pub fn new(config: Config) -> Self { PluginMethod::new(config) }
}

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) { PluginMethod::build(self, app) }
}
