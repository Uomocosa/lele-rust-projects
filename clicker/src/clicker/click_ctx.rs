use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

use crate::clicker;

#[derive(bevy::ecs::system::SystemParam)]
pub struct ClickCtx<'w, 's> {
    pub targets: Query<'w, 's, (&'static clicker::Owner, &'static mut clicker::ClickCounter)>,
    pub global: ResMut<'w, clicker::GlobalCounter>,
    pub commands: ResMut<'w, p2p::Commands<clicker::ClickDelta>>,
    pub roster: Res<'w, roster::Roster>,
    pub lobby: Res<'w, clicker::ActiveLobby>,
    pub own: Res<'w, net_id::NetworkId>,
}

#[cfg(test)]
mod tests {
    use super::ClickCtx;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    use crate::clicker;

    fn probe(_ctx: ClickCtx) {}

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Commands::<clicker::ClickDelta>::default());
        app.insert_resource(roster::Roster::default());
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter::default(),
        ));
        app.add_systems(Update, probe);
        app.update();
    }
}
