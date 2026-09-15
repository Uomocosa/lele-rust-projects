use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;
use crate::constants;

pub fn request_snapshot(
    time: Res<Time>,
    mut requested: Local<bool>,
    mut last: Local<f64>,
    lobby: Res<clicker::ActiveLobby>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
) {
    let time = time.into_inner();
    let now = time.elapsed().as_secs_f64();
    if *requested && now - *last < clicker::SNAPSHOT_REFETCH_SECS {
        return;
    }
    *requested = true;
    *last = now;
    let lobby = lobby.into_inner();
    let commands = commands.into_inner();
    commands.push(p2p::Command::FetchHistory {
        lobby: (**lobby).clone(),
        chunk: constants::SNAPSHOT_CHUNK,
    });
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::request_snapshot;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::p2p;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.add_systems(Update, request_snapshot);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
    }
}
