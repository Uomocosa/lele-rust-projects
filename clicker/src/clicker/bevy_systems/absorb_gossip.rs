use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::clicker;

pub fn absorb_gossip(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    mut targets: Query<(&clicker::Owner, &mut clicker::TargetPos)>,
    lobby: Res<clicker::ActiveLobby>,
    own: Res<net_id::NetworkId>,
    mut seen: Local<bool>,
) {
    let lobby = lobby.into_inner();
    let own = own.into_inner();
    let topic = clicker::pos_topic(lobby);
    let mut rest = Vec::new();
    for event in events.take_all() {
        match event {
            p2p::Event::Gossip {
                topic: incoming,
                from,
                data,
            } if incoming == topic => {
                let sender = net_id::NetworkId::from_peer(&from);
                if sender == *own {
                    continue;
                }
                let Ok(msg) = bincode::deserialize::<clicker::CursorMsg>(&data) else {
                    continue;
                };
                let clicker::CursorMsg::Move { pos, .. } = msg else {
                    continue;
                };
                let [x, y] = pos;
                for (owner, mut target) in &mut targets {
                    if ***owner == *sender {
                        **target = Vec2::new(x, y);
                        if !*seen {
                            *seen = true;
                            tracing::info!("first remote pos from={from} x={x} y={y}");
                        }
                        break;
                    }
                }
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::absorb_gossip;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(99));
        let sender = net_id::NetworkId::from_peer("peer");
        let target = app
            .world_mut()
            .spawn((clicker::Owner(sender), clicker::TargetPos(Vec2::ZERO)))
            .id();
        let msg = clicker::CursorMsg::Move {
            owner: sender,
            pos: [30.0, 40.0],
        };
        let data = bincode::serialize(&msg).unwrap_or_default();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Gossip {
                topic: "clicker/alpha/pos".to_string(),
                from: "peer".to_string(),
                data,
            });
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Gossip {
                topic: "clicker/other/pos".to_string(),
                from: "peer".to_string(),
                data: Vec::new(),
            });
        app.add_systems(Update, absorb_gossip);
        app.update();
        let pos = app.world().get::<clicker::TargetPos>(target).unwrap();
        assert!((pos.x - 30.0).abs() < 0.001);
        assert!((pos.y - 40.0).abs() < 0.001);
        assert_eq!(
            app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .len(),
            1
        );
    }
}
