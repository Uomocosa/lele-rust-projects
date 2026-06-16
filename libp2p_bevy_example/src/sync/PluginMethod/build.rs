use bevy::prelude::*;

use crate::sync::resource;
use crate::sync::system;
use crate::sync::Plugin;

pub fn build(_plugin: &Plugin, app: &mut App) {
    app.init_resource::<resource::NetworkState>()
        .init_resource::<resource::Tick>()
        .init_resource::<resource::RemoteInputBuffer>()
        .add_systems(FixedUpdate, system::tick)
        .add_systems(FixedUpdate, system::broadcast)
        .add_systems(FixedUpdate, system::apply_remote_inputs);
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;
    use crate::p2p::Plugin;
    use crate::sync;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((Plugin::new(Config::default()), sync::Plugin));
    }
}
