use crate::lobby;

pub fn rooms<'a>(ctx: &'a mut lobby::bevy_systems::JoinCtx<'_>) -> lobby::JoinRooms<'a> {
    lobby::JoinRooms {
        active: &mut ctx.active,
        selected: &mut ctx.selected,
        roster_lobby: &mut ctx.roster_lobby,
        pending: &mut ctx.pending,
        gate: &mut ctx.gate,
    }
}

#[cfg(test)]
mod tests {
    use super::rooms;
    use crate::lobby;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::roster;

    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(lobby::SelectedRoom(Some("alpha".to_string())));
        app.insert_resource(roster::Lobby("alpha".to_string()));
        app.insert_resource(lobby::JoinPending(Some("alpha".to_string())));
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(lobby::DirectoryLive(true));
        let mut params = SystemState::<lobby::bevy_systems::JoinCtx>::new(app.world_mut());
        let mut ctx = params.get_mut(app.world_mut()).unwrap();
        let bundled = rooms(&mut ctx);
        assert_eq!(*bundled.active, clicker::ActiveLobby("alpha".to_string()));
        assert_eq!(
            *bundled.pending,
            lobby::JoinPending(Some("alpha".to_string()))
        );
    }
}
