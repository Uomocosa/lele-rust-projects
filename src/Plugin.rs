use bevy::prelude::*;

use crate::PluginMethod;

pub struct Plugin {
    pub data_dir: std::path::PathBuf,
}

impl Default for Plugin {
    fn default() -> Self {
        Self {
            data_dir: std::path::PathBuf::from("freenet_data"),
        }
    }
}

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        PluginMethod::build(self, app);
    }
}
