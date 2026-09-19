use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use super::spawn_on_join_ctx::SpawnOnJoinCtx;
use super::spawn_on_join_ctx_held;
use crate::clicker;

pub fn spawn_on_join(
    mut commands: Commands,
    owners: Query<&clicker::Owner>,
    ctx: SpawnOnJoinCtx,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let SpawnOnJoinCtx {
        roster,
        lobby,
        own,
        gate,
    } = ctx;
    let roster = roster.into_inner();
    let lobby = lobby.into_inner();
    let own = own.into_inner();
    let mut known = Vec::new();
    for owner in &owners {
        known.push(**owner);
    }
    let Some(members) = roster.get(&**lobby) else {
        return;
    };
    for peer in members.values() {
        let id = net_id::NetworkId::from_peer(peer);
        let held = spawn_on_join_ctx_held::held(*own, &gate.pending, &gate.absent, peer, id);
        if id == *own || known.contains(&id) || held {
            continue;
        }
        if known.len() >= clicker::LOBBY_CAP {
            tracing::warn!("lobby {} full, ignoring {}", **lobby, *id);
            continue;
        }
        known.push(id);
        tracing::info!("accounting for remote owner={} peer={peer}", *id);
        tracing::info!("cursor pending owner={} peer={peer}", *id);
        let spot = clicker::spawn_spot(id);
        let pending = Color::srgb(0.5, 0.5, 0.5);
        let fill = commands
            .spawn((
                clicker::CursorIcon,
                clicker::Owner(id),
                clicker::CursorColor(pending),
                clicker::ClickCounter::default(),
                clicker::TargetPos(spot),
                Mesh2d(meshes.add(clicker::cursor_mesh(1.0))),
                MeshMaterial2d(materials.add(pending)),
                Transform::from_translation(Vec3::new(spot.x, spot.y, 10.0)),
            ))
            .id();
        commands.entity(fill).with_children(|parent| {
            parent.spawn((
                clicker::CursorLabel,
                Text2d::new(clicker::math_formatter(0)),
                Transform::from_translation(Vec3::new(-17.0, 34.0, 0.5)),
            ));
        });
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_on_join;
    use crate::clicker;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::{net_id, roster};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(roster::Roster::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(lobby::JoinPending::default());
        app.world_mut().resource_mut::<roster::Roster>().add_entry(
            "alpha".to_string(),
            *blake3::hash(b"peer").as_bytes(),
            "peer".to_string(),
        );
        app.add_systems(Update, spawn_on_join);
        app.update();
        app.update();
        let count = app
            .world_mut()
            .query::<&clicker::Owner>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
        let mut materials = app.world_mut().query::<&MeshMaterial2d<ColorMaterial>>();
        let handle = materials.iter(app.world()).next().unwrap().clone();
        let material = app
            .world()
            .resource::<Assets<ColorMaterial>>()
            .get(&handle)
            .unwrap();
        assert_eq!(material.color, Color::srgb(0.5, 0.5, 0.5));
    }

    #[test]
    fn placeholder_has_no_player_no() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(roster::Roster::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(lobby::JoinPending::default());
        app.world_mut().resource_mut::<roster::Roster>().add_entry(
            "alpha".to_string(),
            *blake3::hash(b"peer").as_bytes(),
            "peer".to_string(),
        );
        app.add_systems(Update, spawn_on_join);
        app.update();
        let numbered = app
            .world_mut()
            .query::<&clicker::PlayerNo>()
            .iter(app.world())
            .count();
        assert_eq!(numbered, 0);
    }

    #[test]
    fn spectate_spawns_while_join_pending() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(roster::Roster::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(lobby::JoinPending(Some("alpha".to_string())));
        app.world_mut().resource_mut::<roster::Roster>().add_entry(
            "alpha".to_string(),
            *blake3::hash(b"peer").as_bytes(),
            "peer".to_string(),
        );
        app.add_systems(Update, spawn_on_join);
        app.update();
        app.update();
        let count = app
            .world_mut()
            .query::<&clicker::Owner>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "spectating joiner renders the room while loading");
    }

    #[test]
    fn absent_peer_not_respawned() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(roster::Roster::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut().resource_mut::<roster::Roster>().add_entry(
            "alpha".to_string(),
            *blake3::hash(b"peer").as_bytes(),
            "peer".to_string(),
        );
        app.world_mut()
            .resource_mut::<lobby::JoinGate>()
            .absent
            .push("peer".to_string());
        app.add_systems(Update, spawn_on_join);
        app.update();
        app.update();
        let count = app
            .world_mut()
            .query::<&clicker::Owner>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "a peer marked absent is never respawned");
    }
}
