use bevy::prelude::*;

use crate::clicker;

pub fn auto_tick(
    auto: Res<clicker::AutoClick>,
    time: Res<Time>,
    mut last: Local<f64>,
    mut ctx: clicker::ClickCtx,
) {
    let auto = auto.into_inner();
    let time = time.into_inner();
    if !**auto {
        return;
    }
    let now = time.elapsed().as_secs_f64();
    if now - *last < 1.0 {
        return;
    }
    *last = now;
    let own = *ctx.own;
    for (owner, mut counter) in &mut ctx.targets {
        if **owner == own {
            clicker::click(
                &mut counter,
                &mut ctx.global,
                &mut ctx.commands,
                &ctx.roster,
                &ctx.lobby,
                own,
            );
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::auto_tick;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(clicker::AutoClick(true));
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::ActiveLobby::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(roster::Roster::default());
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter::default(),
        ));
        app.add_systems(Update, auto_tick);
        app.update();
    }
}
