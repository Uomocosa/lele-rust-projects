use bevy::prelude::App;
use derive_more::Deref;

use super::plugin_build;
use crate::roster;

#[derive(Deref)]
pub struct Plugin(pub roster::Config);

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) { plugin_build::build(self, app) }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::Plugin;
    use crate::roster;

    #[test]
    fn test_usage() {
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(Plugin(roster::Config::new(rx)));
        app.update();

        assert!(app.world().get_resource::<roster::Roster>().is_some());
    }
}
