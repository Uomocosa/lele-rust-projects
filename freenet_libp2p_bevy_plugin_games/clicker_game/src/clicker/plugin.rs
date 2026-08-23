use bevy::prelude::App;

use super::plugin_build;

pub struct Plugin;

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) { plugin_build::build(app) }
}

#[cfg(test)]
mod tests {
    use super::Plugin;

    #[test]
    fn test_usage() {
        let _plugin = Plugin;
    }
}
