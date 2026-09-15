use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

pub fn credit_click(
    targets: &mut Query<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    pending: &mut clicker::PendingClicks,
    own: net_id::NetworkId,
    sender: net_id::NetworkId,
    owner: net_id::NetworkId,
    delta: i32,
) {
    if owner == own {
        return;
    }
    if credit_logical(targets, owner, delta) {
        return;
    }
    if credit_sender(targets, sender, delta) {
        pending.items.push(clicker::PendingClick {
            sender,
            owner,
            delta: 0,
            absolute: false,
        });
        return;
    }
    pending.items.push(clicker::PendingClick {
        sender,
        owner,
        delta,
        absolute: false,
    });
}

// needed helper: credits slots keyed by logical player id
fn credit_logical(
    targets: &mut Query<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    owner: net_id::NetworkId,
    delta: i32,
) -> bool {
    for (slot_owner, player, mut counter) in targets {
        if ***slot_owner == *owner || player.is_some_and(|p| **p == *owner) {
            counter.add(delta);
            return true;
        }
    }
    false
}

// needed helper: credits the transport sender slot for unresolved remotes
fn credit_sender(
    targets: &mut Query<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    sender: net_id::NetworkId,
    delta: i32,
) -> bool {
    for (slot_owner, _, mut counter) in targets {
        if ***slot_owner == *sender {
            counter.add(delta);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::credit_click;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    fn credit_once(
        mut targets: Query<(
            &clicker::Owner,
            Option<&clicker::PlayerNo>,
            &mut clicker::ClickCounter,
        )>,
        mut pending: ResMut<clicker::PendingClicks>,
    ) {
        let sender = net_id::NetworkId::from_peer("peer");
        credit_click(
            &mut targets,
            &mut pending,
            net_id::NetworkId(99),
            sender,
            sender,
            3,
        );
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(clicker::PendingClicks::default());
        let sender = net_id::NetworkId::from_peer("peer");
        let target = app
            .world_mut()
            .spawn((clicker::Owner(sender), clicker::ClickCounter::default()))
            .id();
        app.add_systems(Update, credit_once);
        app.update();
        assert_eq!(
            **app.world().get::<clicker::ClickCounter>(target).unwrap(),
            3
        );
        assert_eq!(
            app.world().resource::<clicker::PendingClicks>().items.len(),
            0
        );
    }
}
