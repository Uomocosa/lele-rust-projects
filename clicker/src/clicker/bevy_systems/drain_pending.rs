use bevy::prelude::*;

use crate::clicker;

pub fn drain_pending(
    mut commands: Commands,
    mut targets: Query<(
        Entity,
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    mut pending: ResMut<clicker::PendingClicks>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut kept = Vec::new();
    for item in &pending.items {
        if !drain_item(&mut commands, &mut targets, &mut materials, item) {
            kept.push(*item);
        }
    }
    pending.items = kept;
}

// needed helper: applies one pending click to the matching slot
fn drain_item(
    commands: &mut Commands,
    targets: &mut Query<(
        Entity,
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    materials: &mut Assets<ColorMaterial>,
    item: &clicker::PendingClick,
) -> bool {
    let found = find_slot(targets, item);
    let Some((entity, credit)) = found else {
        return false;
    };
    if item.absolute {
        if let Ok((_, _, _, mut counter)) = targets.get_mut(entity) {
            counter.max(item.delta);
        }
    } else if credit != 0
        && let Ok((_, _, _, mut counter)) = targets.get_mut(entity)
    {
        counter.add(credit);
    }
    clicker::label_slot(
        commands,
        materials,
        entity,
        &item.sender.to_string(),
        *item.owner,
    );
    true
}

// needed helper: locates the slot for a pending click without holding borrows
fn find_slot(
    targets: &Query<(
        Entity,
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    item: &clicker::PendingClick,
) -> Option<(Entity, i32)> {
    let mut sender = None;
    for (entity, owner, player, _) in targets {
        if ***owner == *item.owner || player.is_some_and(|p| **p == *item.owner) {
            return Some((entity, item.delta));
        }
        if !item.absolute && ***owner == *item.sender && player.is_none() {
            sender = Some(entity);
        }
    }
    sender.map(|entity| (entity, item.delta))
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::drain_pending;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(clicker::PendingClicks {
            items: vec![clicker::PendingClick {
                sender: net_id::NetworkId::from_peer("peer"),
                owner: net_id::NetworkId(5),
                delta: 4,
                absolute: false,
            }],
            parked: 1,
        });
        let sender = net_id::NetworkId::from_peer("peer");
        let target = app
            .world_mut()
            .spawn((clicker::Owner(sender), clicker::ClickCounter::default()))
            .id();
        app.add_systems(Update, drain_pending);
        app.update();
        app.update();
        assert_eq!(
            **app.world().get::<clicker::ClickCounter>(target).unwrap(),
            4
        );
        assert_eq!(**app.world().get::<clicker::PlayerNo>(target).unwrap(), 5);
        assert_eq!(
            app.world().resource::<clicker::PendingClicks>().items.len(),
            0
        );
    }
}
