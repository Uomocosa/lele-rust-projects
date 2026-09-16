use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;

pub fn send_sync_req(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
    own: Res<net_id::NetworkId>,
    mut live: Local<std::collections::HashSet<String, std::hash::RandomState>>,
) {
    let own = own.into_inner();
    let commands = commands.into_inner();
    let mut rest = Vec::new();
    for event in events.take_all() {
        match event {
            p2p::Event::PeerConnected(peer) => {
                if live.insert(peer.clone()) {
                    commands.push(p2p::Command::Send {
                        peer_id: peer.clone(),
                        payload: clicker::CursorMsg::SyncReq { requester: *own },
                    });
                }
                rest.push(p2p::Event::PeerConnected(peer));
            }
            p2p::Event::PeerDisconnected(peer) => {
                live.remove(peer.as_str());
                rest.push(p2p::Event::PeerDisconnected(peer));
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::send_sync_req;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::PeerConnected("peer".to_string()));
        app.add_systems(Update, send_sync_req);
        app.update();
        assert_eq!(
            app.world()
                .resource::<p2p::Commands<clicker::CursorMsg>>()
                .len(),
            1
        );
        assert_eq!(
            app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .len(),
            1,
            "the connect event passes downstream to the roster"
        );
    }

    #[test]
    fn duplicate_connect_requests_once() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(net_id::NetworkId(1));
        app.add_systems(Update, send_sync_req);
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<p2p::Events<clicker::CursorMsg>>()
                .push(p2p::Event::PeerConnected("peer".to_string()));
            app.update();
        }
        assert_eq!(
            app.world()
                .resource::<p2p::Commands<clicker::CursorMsg>>()
                .len(),
            1,
            "one live peer draws exactly one sync request"
        );
    }

    #[test]
    fn reconnect_after_disconnect_requests_again() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(net_id::NetworkId(1));
        app.add_systems(Update, send_sync_req);
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::PeerConnected("peer".to_string()));
        app.update();
        drain(&mut app);
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::PeerDisconnected("peer".to_string()));
        app.update();
        drain(&mut app);
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::PeerConnected("peer".to_string()));
        app.update();
        assert_eq!(
            app.world()
                .resource::<p2p::Commands<clicker::CursorMsg>>()
                .len(),
            2,
            "a genuine reconnect draws a fresh sync request"
        );
    }

    // needed helper: simulates downstream event consumption between updates
    fn drain(app: &mut App) {
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .take_all();
    }
}
