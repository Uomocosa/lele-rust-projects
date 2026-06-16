use bevy::prelude::*;

use crate::clicker::component;
use crate::p2p::resource::PeerState;

pub fn update_counter(
    p2p_state: Res<PeerState>,
    mut query: Query<(&component::Owner, &component::ClickCounter, &mut Text)>,
) {
    for (owner, counter, mut text) in &mut query {
        let label = if owner.peer_id == p2p_state.local_peer_id {
            "You"
        } else {
            "Opponent"
        };
        text.0 = format!("{}: {}", label, counter.count);
    }
}

#[cfg(test)]
mod tests {
    use crate::clicker::component;
    use crate::clicker::system;
    use crate::p2p::resource::PeerState;
    use crate::p2p::Config;
    use bevy::ecs::schedule::Schedule;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut world = World::new();

        let local_peer = libp2p::PeerId::random();
        let remote_peer = libp2p::PeerId::random();

        world.spawn((
            component::Owner {
                peer_id: local_peer,
            },
            component::ClickCounter { count: 5 },
            Text::new("You: 0"),
        ));
        world.spawn((
            component::Owner {
                peer_id: remote_peer,
            },
            component::ClickCounter { count: 3 },
            Text::new("Opponent: 0"),
        ));

        let p2p_state = PeerState::new(&Config::default(), local_peer);
        world.insert_resource(p2p_state);

        let mut schedule = Schedule::default();
        schedule.add_systems(system::update_counter);
        schedule.run(&mut world);

        let texts: Vec<_> = world.query::<&Text>().iter(&world).collect();
        assert_eq!(texts[0].0, "You: 5");
        assert_eq!(texts[1].0, "Opponent: 3");
    }
}
