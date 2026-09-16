use crate::clicker;

pub fn gate_open(ctx: &clicker::bevy_systems::SpawnCtx) -> bool {
    ctx.pending.is_none()
}

#[cfg(test)]
mod tests {
    use super::gate_open;
    use crate::clicker;
    use crate::lobby;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{net_id, roster};

    fn ctx_app(pending: lobby::JoinPending) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(roster::Roster::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(pending);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = ctx_app(lobby::JoinPending::default());
        let mut params = SystemState::<clicker::bevy_systems::SpawnCtx>::new(app.world_mut());
        let ctx = params.get_mut(app.world_mut()).unwrap();
        assert!(gate_open(&ctx), "no pending join leaves the gate open");
        let mut app = ctx_app(lobby::JoinPending(Some("alpha".to_string())));
        let mut params = SystemState::<clicker::bevy_systems::SpawnCtx>::new(app.world_mut());
        let ctx = params.get_mut(app.world_mut()).unwrap();
        assert!(!gate_open(&ctx), "pending join closes the gate");
    }
}
