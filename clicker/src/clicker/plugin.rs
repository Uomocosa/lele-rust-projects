use super::plugin_build;
use bevy::prelude::App;

pub struct Plugin;
#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) { plugin_build::build(self, app) }
}
#[cfg(test)]
mod tests {
    use super::Plugin;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{p2p, roster};

    use crate::clicker;
    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Commands::<clicker::ClickDelta>::default());
        app.insert_resource(p2p::Events::<clicker::ClickDelta>::default());
        app.insert_resource(roster::Roster::default());
        app.add_plugins(Plugin);
        app.update();
    }
}
