use bevy::prelude::*;

use crate::clicker;

pub fn tick_log(
    query: Query<(&clicker::Owner, &clicker::ClickCounter)>,
    logical: Query<(&clicker::PlayerNo, &clicker::ClickCounter)>,
    global: Res<clicker::GlobalCounter>,
    lobby: Res<clicker::ActiveLobby>,
    time: Res<Time>,
    mut last: Local<f64>,
) {
    let time = time.into_inner();
    let now = time.elapsed().as_secs_f64();
    if now - *last < 1.0 {
        return;
    }
    *last = now;
    let global = global.into_inner();
    let lobby = lobby.into_inner();
    for (owner, counter) in &query {
        tracing::info!(
            "tick lobby={} owner={} count={} global={}",
            **lobby,
            ***owner,
            **counter,
            **global
        );
    }
    let mut p1 = 0;
    let mut p2 = 0;
    let mut p3 = 0;
    for (player, counter) in &logical {
        if **player == 1 {
            p1 = **counter;
        } else if **player == 2 {
            p2 = **counter;
        } else if **player == 3 {
            p3 = **counter;
        }
    }
    tracing::info!(
        "sync lobby={} p1={p1} p2={p2} p3={p3} global={}",
        **lobby,
        **global
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::tick_log;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter::default(),
        ));
        app.add_systems(Update, tick_log);
        app.update();
    }
}
