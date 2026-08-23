use bevy::prelude::*;

use crate::clicker;
use freenet_libp2p_bevy_plugin::net_id;

pub fn setup(mut commands: Commands, identity: Res<net_id::LocalIdentity>) {
    commands.spawn(Camera2d);
    clicker::spawn_target(&mut commands, **identity, 0, true);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::setup;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(net_id::LocalIdentity(net_id::NetworkId(9)));
        app.add_systems(Update, setup);
        app.update();

        let mut query = app.world_mut().query::<&clicker::Owner>();
        let owners: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(owners.len(), 1);
        assert_eq!(**owners[0], net_id::NetworkId(9));
    }
}
