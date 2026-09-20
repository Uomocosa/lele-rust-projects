use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::roster;

use crate::clicker;

pub fn despawn_on_leave(
    mut commands: Commands,
    roster: Res<roster::Roster>,
    query: Query<(
        Entity,
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &clicker::ClickCounter,
    )>,
    mut ctx: clicker::bevy_systems::LeaveCtx,
) {
    let roster = roster.into_inner();
    let room = (**ctx.lobby).clone();
    let own = *ctx.own;
    let mut live = Vec::new();
    if let Some(members) = roster.get(&room) {
        for peer in members.values() {
            live.push(net_id::NetworkId::from_peer(peer));
        }
    }
    for (entity, owner, player, counter) in &query {
        if **owner != own && !live.contains(&**owner) {
            if player.is_none() {
                tracing::debug!(target: "clicker", owner = ?owner, "leave: unlabeled slot despawned");
                clicker::DecisionLog::record(&format!(
                    "leave: unlabeled slot despawned owner={owner:?}"
                ));
                commands.entity(entity).despawn();
                continue;
            }
            if ctx.absence_secs(***owner) < clicker::LEAVE_GRACE_SECS {
                tracing::debug!(target: "clicker", owner = ?owner, "leave: labeled slot in grace");
                clicker::DecisionLog::record(&format!(
                    "leave: labeled slot in grace owner={owner:?}"
                ));
                continue;
            }
            ctx.absent.remove(&***owner);
            if let Some(numbered) = player {
                ctx.tombstones.keep(**numbered, **counter);
            }
            tracing::debug!(target: "clicker", owner = ?owner, "leave: labeled slot grace expired, despawned");
            clicker::DecisionLog::record(&format!(
                "leave: labeled slot grace expired, despawned owner={owner:?}"
            ));
            commands.entity(entity).despawn();
        }
    }
    ctx.absent
        .retain(|owner, _| !live.iter().any(|id| **id == *owner));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::despawn_on_leave;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, roster};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(roster::Roster::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::ScoreTombstones::default());
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter::default(),
        ));
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(9)),
            clicker::ClickCounter::default(),
        ));
        app.add_systems(Update, despawn_on_leave);
        app.update();
        app.update();
        let count = app
            .world_mut()
            .query::<&clicker::Owner>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn labeled_slot_survives_transient_roster_absence() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(roster::Roster::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::ScoreTombstones::default());
        let slot = app
            .world_mut()
            .spawn((
                clicker::Owner(net_id::NetworkId(9)),
                clicker::PlayerNo(3),
                clicker::ClickCounter(7),
            ))
            .id();
        app.add_systems(Update, despawn_on_leave);
        app.update();
        assert!(
            app.world().get_entity(slot).is_ok(),
            "labeled slot survives transient roster absence"
        );
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs(1),
        ));
        for _ in 0..70 {
            app.update();
        }
        assert!(
            app.world().get_entity(slot).is_err(),
            "labeled slot despawned after sustained absence"
        );
    }
}
