use bevy::prelude::App;
use derive_more::Deref;

use super::plugin_build;
use crate::boxes;

#[derive(Deref)]
pub struct Plugin(pub boxes::Config);

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) { plugin_build::build(self, app) }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::Plugin;
    use crate::boxes;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_plugins(Plugin(boxes::Config::new(boxes::PlayerId(1))));
        app.update();

        let config = app.world().get_resource::<boxes::Config>();
        assert_eq!(config.map(|c| **c), Some(boxes::PlayerId(1)));
    }
}
