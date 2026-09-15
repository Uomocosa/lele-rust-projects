use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;

pub fn subscribe_roster(
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
    lobby: Res<clicker::ActiveLobby>,
) {
    let lobby = lobby.into_inner();
    let commands = commands.into_inner();
    commands.push(p2p::Command::Subscribe {
        topic: clicker::gossip_roster_topic(lobby),
    });
    commands.push(p2p::Command::Subscribe {
        topic: clicker::click_topic(lobby),
    });
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::subscribe_roster;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::p2p;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.add_systems(Startup, subscribe_roster);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 2);
    }
}
