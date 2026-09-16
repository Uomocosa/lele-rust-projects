use crate::clicker;
use crate::lobby;
use bevy::prelude::*;
pub fn build(_plugin: &clicker::Plugin, app: &mut App) {
    app.init_resource::<clicker::ScoreTombstones>();
    app.init_resource::<lobby::JoinPending>();
    app.init_resource::<lobby::JoinGate>();
    app.add_systems(Startup, clicker::bevy_systems::setup)
        .add_systems(Startup, clicker::bevy_systems::spawn_cursor)
        .add_systems(Startup, clicker::bevy_systems::log_connected)
        .add_systems(Startup, clicker::bevy_systems::request_snapshot)
        .add_systems(Startup, clicker::bevy_systems::subscribe_pos)
        .add_systems(Startup, clicker::bevy_systems::subscribe_roster)
        .add_systems(Update, clicker::bevy_systems::detect_click)
        .add_systems(Update, clicker::bevy_systems::follow_mouse)
        .add_systems(Update, clicker::bevy_systems::publish_cursor)
        .add_systems(
            Update,
            (
                clicker::bevy_systems::send_sync_req,
                clicker::bevy_systems::send_sync_heartbeat,
                clicker::bevy_systems::spawn_on_join,
                clicker::bevy_systems::drain_pending,
                clicker::bevy_systems::resolve_player,
                clicker::bevy_systems::absorb_gossip,
                clicker::bevy_systems::absorb_click_gossip,
                clicker::bevy_systems::absorb_roster,
                clicker::bevy_systems::absorb_sync,
                clicker::bevy_systems::apply_delta,
                clicker::bevy_systems::absorb_snapshot,
                clicker::bevy_systems::sync_global,
            )
                .chain(),
        )
        .add_systems(Update, clicker::bevy_systems::ease_remote)
        .add_systems(Update, clicker::bevy_systems::despawn_on_leave)
        .add_systems(
            Update,
            clicker::bevy_systems::publish_snapshot.run_if(clicker::bevy_systems::publish_due),
        )
        .add_systems(Update, clicker::bevy_systems::request_snapshot)
        .add_systems(Update, clicker::bevy_systems::emit_flash)
        .add_systems(Update, clicker::bevy_systems::animate_flash)
        .add_systems(Update, clicker::bevy_systems::update_total_board)
        .add_systems(Update, clicker::bevy_systems::update_cursor_label)
        .add_systems(Update, clicker::bevy_systems::update_score)
        .add_systems(Update, clicker::bevy_systems::tick_log)
        .add_systems(Update, clicker::bevy_systems::pos_log);
}

#[cfg(test)]
mod tests {
    use super::build;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(ButtonInput::<MouseButton>::default());
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(roster::Roster::default());
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(clicker::InstanceInfo {
            namespace: "test".to_string(),
            instance_tag: 0,
            own_id: net_id::NetworkId(1),
        });
        build(&clicker::Plugin, &mut app);
        app.update();
    }
}
