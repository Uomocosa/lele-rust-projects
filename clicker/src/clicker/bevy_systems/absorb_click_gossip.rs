use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::clicker;

pub fn absorb_click_gossip(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    mut targets: Query<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    mut pending: ResMut<clicker::PendingClicks>,
    lobby: Res<clicker::ActiveLobby>,
    own: Res<net_id::NetworkId>,
) {
    let lobby = lobby.into_inner();
    let own = *own.into_inner();
    let topic = clicker::click_topic(lobby);
    let mut rest = Vec::new();
    for event in events.take_all() {
        match event {
            p2p::Event::Gossip {
                topic: incoming,
                from,
                data,
            } if incoming == topic => {
                let sender = net_id::NetworkId::from_peer(&from);
                if sender == own {
                    continue;
                }
                let Ok(msg) = bincode::deserialize::<clicker::CursorMsg>(&data) else {
                    continue;
                };
                let clicker::CursorMsg::Click { owner, delta } = msg else {
                    continue;
                };
                clicker::credit_click(&mut targets, &mut pending, own, sender, owner, delta);
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::absorb_click_gossip;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    fn gossip_click(from: &str, owner: u64, delta: i32) -> p2p::Event<clicker::CursorMsg> {
        let msg = clicker::CursorMsg::Click {
            owner: net_id::NetworkId(owner),
            delta,
        };
        p2p::Event::Gossip {
            topic: clicker::click_topic(&clicker::ActiveLobby("alpha".to_string())),
            from: from.to_string(),
            data: bincode::serialize(&msg).unwrap_or_default(),
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        let sender = net_id::NetworkId::from_peer("peer-2");
        let target = app
            .world_mut()
            .spawn((clicker::Owner(sender), clicker::ClickCounter::default()))
            .id();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(gossip_click("peer-2", 2, 4));
        app.add_systems(Update, absorb_click_gossip);
        app.update();
        assert_eq!(
            **app.world().get::<clicker::ClickCounter>(target).unwrap(),
            4
        );
    }

    #[test]
    fn test_wrong_topic_preserved() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        let msg = clicker::CursorMsg::Click {
            owner: net_id::NetworkId(2),
            delta: 4,
        };
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Gossip {
                topic: "other".to_string(),
                from: "peer-2".to_string(),
                data: bincode::serialize(&msg).unwrap_or_default(),
            });
        app.add_systems(Update, absorb_click_gossip);
        app.update();
        assert_eq!(
            app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .len(),
            1
        );
    }
}
