use bevy::prelude::*;

use super::plugin_build;
use crate::clicker;

pub struct Plugin {
    pub config: clicker::Config,
}

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        plugin_build::build(self, app)
    }
}
