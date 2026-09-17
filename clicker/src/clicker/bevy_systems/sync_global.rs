use std::collections::HashSet;

use bevy::prelude::*;

use crate::clicker;

pub fn sync_global(
    targets: Query<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &clicker::ClickCounter,
    )>,
    mut global: ResMut<clicker::GlobalCounter>,
    tombstones: Res<clicker::ScoreTombstones>,
    mut seen: Local<HashSet<u64, std::hash::RandomState>>,
) {
    let tombstones = tombstones.into_inner();
    if tombstones.is_empty() {
        seen.clear();
    }
    for (_, player, _) in &targets {
        if let Some(number) = player {
            seen.insert(**number);
        }
    }
    let mut total: i32 = 0;
    for (_, player, counter) in &targets {
        let Some(number) = player else {
            continue;
        };
        let mut value = **counter;
        if let Some(saved) = tombstones.get(&**number) {
            value = value.max(*saved);
        }
        total = total.saturating_add(value);
    }
    for (logical, count) in tombstones.iter() {
        if !seen.contains(logical) {
            continue;
        }
        let live = targets
            .iter()
            .any(|(_, player, _)| player.is_some_and(|number| **number == *logical));
        if !live {
            total = total.saturating_add(*count);
        }
    }
    if **global != total {
        **global = total;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::sync_global;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    fn counter_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(clicker::GlobalCounter(999));
        app.insert_resource(clicker::ScoreTombstones::default());
        app
    }

    #[test]
    fn test_usage() {
        let mut app = counter_app();
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::PlayerNo(1),
            clicker::ClickCounter(4),
        ));
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(2)),
            clicker::PlayerNo(2),
            clicker::ClickCounter(5),
        ));
        app.add_systems(Update, sync_global);
        app.update();
        assert_eq!(**app.world().resource::<clicker::GlobalCounter>(), 9);
    }

    #[test]
    fn leaver_score_retained() {
        let mut app = counter_app();
        let slot = app
            .world_mut()
            .spawn((
                clicker::Owner(net_id::NetworkId(9)),
                clicker::PlayerNo(9),
                clicker::ClickCounter(5),
            ))
            .id();
        app.add_systems(Update, sync_global);
        app.update();
        assert_eq!(**app.world().resource::<clicker::GlobalCounter>(), 5);
        app.world_mut()
            .resource_mut::<clicker::ScoreTombstones>()
            .keep(9, 5);
        app.world_mut().despawn(slot);
        app.update();
        assert_eq!(
            **app.world().resource::<clicker::GlobalCounter>(),
            5,
            "retained score of a departed player stays in the room total"
        );
    }

    #[test]
    fn never_seen_tombstone_excluded() {
        let mut app = counter_app();
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::PlayerNo(1),
            clicker::ClickCounter(4),
        ));
        app.world_mut()
            .resource_mut::<clicker::ScoreTombstones>()
            .keep(9, 5);
        app.add_systems(Update, sync_global);
        app.update();
        assert_eq!(
            **app.world().resource::<clicker::GlobalCounter>(),
            4,
            "snapshot claims for never-labeled players never inflate the total"
        );
    }

    #[test]
    fn overlap_takes_max() {
        let mut app = counter_app();
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(2)),
            clicker::PlayerNo(2),
            clicker::ClickCounter(3),
        ));
        app.world_mut()
            .resource_mut::<clicker::ScoreTombstones>()
            .keep(2, 7);
        app.add_systems(Update, sync_global);
        app.update();
        assert_eq!(
            **app.world().resource::<clicker::GlobalCounter>(),
            7,
            "stale tombstone beats a not-yet-restored live slot"
        );
    }

    #[test]
    fn unlabeled_count_never_counts() {
        let mut app = counter_app();
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::PlayerNo(1),
            clicker::ClickCounter(4),
        ));
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(9)),
            clicker::ClickCounter(16),
        ));
        app.add_systems(Update, sync_global);
        app.update();
        assert_eq!(
            **app.world().resource::<clicker::GlobalCounter>(),
            4,
            "merged-but-unresolved slots join the total only once labeled"
        );
    }

    #[test]
    fn live_above_tombstone_wins() {
        let mut app = counter_app();
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(2)),
            clicker::PlayerNo(2),
            clicker::ClickCounter(9),
        ));
        app.world_mut()
            .resource_mut::<clicker::ScoreTombstones>()
            .keep(2, 7);
        app.add_systems(Update, sync_global);
        app.update();
        assert_eq!(
            **app.world().resource::<clicker::GlobalCounter>(),
            9,
            "no double count when the live slot already leads"
        );
    }
}
