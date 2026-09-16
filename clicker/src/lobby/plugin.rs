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
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    use crate::clicker;
    use crate::lobby;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_resource::<Time>();
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(roster::Roster::default());
        app.insert_resource(roster::Lobby::default());
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(clicker::ActiveLobby::default());
        let (_dir_tx, dir_rx) = tokio::sync::mpsc::unbounded_channel();
        let (req_tx, _req_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let (_room_tx, room_rx) = tokio::sync::watch::channel(None::<String>);
        app.insert_resource(lobby::DirectoryFeed(std::sync::Mutex::new(Some(dir_rx))));
        app.insert_resource(lobby::RoomRequestTx(req_tx));
        app.insert_resource(lobby::RoomRx(std::sync::Mutex::new(Some(room_rx))));
        app.add_plugins(Plugin);
        app.update();
        assert!(
            app.world()
                .get_resource::<State<lobby::AppState>>()
                .is_some()
        );
    }
}
