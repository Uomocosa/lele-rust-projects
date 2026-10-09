use libp2p::gossipsub;
use libp2p::kad;
use libp2p::mdns;
use libp2p::swarm::behaviour::toggle::Toggle;
use libp2p::swarm::dial_opts::PeerCondition;

use crate::p2p;

pub fn dispatch_net_command<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    room_queries: &mut std::collections::HashMap<libp2p::kad::QueryId, String>,
    command: p2p::NetCommand,
) {
    match command {
        p2p::NetCommand::Dial { peer_id, addrs } => {
            p2p::swarm_loop::dial_peer(
                swarm,
                event_tx,
                &peer_id,
                &addrs,
                PeerCondition::DisconnectedAndNotDialing,
            );
        }
        p2p::NetCommand::DialForce { peer_id, addrs } => {
            p2p::swarm_loop::dial_peer(swarm, event_tx, &peer_id, &addrs, PeerCondition::Always);
        }
        p2p::NetCommand::ReserveRelay { relay_addr } => {
            if let Ok(addr) = relay_addr.parse::<libp2p::Multiaddr>() {
                let _ = swarm.listen_on(addr);
            }
        }
        p2p::NetCommand::SetMdns { enabled } => {
            swarm.behaviour_mut().mdns = Toggle::from(
                enabled
                    .then(|| {
                        mdns::tokio::Behaviour::new(mdns::Config::default(), *swarm.local_peer_id())
                            .ok()
                    })
                    .flatten(),
            );
        }
        p2p::NetCommand::AddKadPeer { peer_id, addrs } => {
            p2p::swarm_loop::seed_kad_peer(swarm, &peer_id, &addrs);
        }
        p2p::NetCommand::ProvideRoom { room } => {
            let _ = swarm
                .behaviour_mut()
                .kademlia
                .start_providing(p2p::provider_key(&room));
        }
        p2p::NetCommand::FindRoom { room } => {
            let id = swarm
                .behaviour_mut()
                .kademlia
                .get_providers(p2p::provider_key(&room));
            room_queries.insert(id, room);
        }
        p2p::NetCommand::PutHistory { room, chunk, data } => {
            let key = p2p::history_key(&room, chunk);
            let record = kad::Record {
                key: key.clone(),
                value: data,
                publisher: None,
                expires: None,
            };
            let _ = swarm
                .behaviour_mut()
                .kademlia
                .put_record(record, kad::Quorum::One);
            let _ = swarm.behaviour_mut().kademlia.start_providing(key);
        }
        p2p::NetCommand::FetchHistory { room, chunk } => {
            let key = p2p::history_key(&room, chunk);
            swarm.behaviour_mut().kademlia.get_record(key);
        }
        p2p::NetCommand::FetchRoster { room } => {
            let _ = swarm
                .behaviour_mut()
                .kademlia
                .start_providing(p2p::provider_key(&room));
            let id = swarm
                .behaviour_mut()
                .kademlia
                .get_providers(p2p::provider_key(&room));
            room_queries.insert(id, room);
        }
        p2p::NetCommand::Subscribe { topic } => {
            let topic = gossipsub::IdentTopic::new(topic);
            let _ = swarm.behaviour_mut().gossipsub.subscribe(&topic);
        }
        p2p::NetCommand::Publish { topic, data } => {
            let topic = gossipsub::IdentTopic::new(topic);
            let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
        }
        p2p::NetCommand::Exchange { peer_id, data } => {
            if let Ok(pid) = peer_id.parse::<libp2p::PeerId>() {
                swarm.behaviour_mut().exchange.send_request(&pid, data);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use libp2p::gossipsub::IdentTopic;
    use libp2p::identity::Keypair;

    use super::dispatch_net_command;
    use crate::p2p;

    fn swarm() -> libp2p::Swarm<p2p::Behaviour<u32>> {
        p2p::build_swarm::<u32>(Keypair::generate_ed25519(), false).unwrap()
    }

    #[tokio::test]
    async fn test_usage() {
        let mut swarm = swarm();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut room_queries = HashMap::new();
        let topic = String::from("lobby/topic");
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut room_queries,
            p2p::NetCommand::Subscribe {
                topic: topic.clone(),
            },
        );
        let hash = IdentTopic::new(topic).hash();
        assert!(swarm.behaviour().gossipsub.topics().any(|t| *t == hash));

        let room = String::from("room-a");
        dispatch_net_command(
            &mut swarm,
            &tx,
            &mut room_queries,
            p2p::NetCommand::FindRoom { room: room.clone() },
        );
        assert_eq!(room_queries.values().collect::<Vec<_>>(), vec![&room]);
    }
}
