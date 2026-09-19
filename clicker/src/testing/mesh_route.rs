use freenet_libp2p_bevy_plugin::p2p;

use super::fake_dht;
use super::mesh::Mesh;
use crate::clicker;

pub fn route(mesh: &mut Mesh) {
    let mut out: Vec<Vec<p2p::Command<clicker::CursorMsg>>> = Vec::new();
    for app in &mut mesh.apps {
        out.push(
            app.world_mut()
                .resource_mut::<p2p::Commands<clicker::CursorMsg>>()
                .take_all(),
        );
    }
    let names: Vec<String> = mesh
        .players
        .iter()
        .map(|player| format!("peer-{}", **player))
        .collect();
    let mut history = std::mem::take(&mut mesh.history);
    let mut pending: Vec<(usize, p2p::Event<clicker::CursorMsg>)> = Vec::new();
    for (from, (name, cmds)) in names.iter().zip(out.iter()).enumerate() {
        for cmd in cmds {
            route_command(mesh, &names, &mut history, from, name, cmd, &mut pending);
        }
    }
    mesh.history = history;
    for (target, app) in mesh.apps.iter_mut().enumerate() {
        let mine: Vec<p2p::Event<clicker::CursorMsg>> = pending
            .iter()
            .filter(|(t, _)| *t == target)
            .map(|(_, e)| e.clone())
            .collect();
        if mine.is_empty() {
            continue;
        }
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .extend(mine);
    }
    pending.clear();
}

// needed helper: translates one outgoing command into routed inbound events
fn route_command(
    mesh: &Mesh,
    names: &[String],
    history: &mut fake_dht::FakeDht,
    from: usize,
    from_name: &str,
    cmd: &p2p::Command<clicker::CursorMsg>,
    pending: &mut Vec<(usize, p2p::Event<clicker::CursorMsg>)>,
) {
    match cmd {
        p2p::Command::Send { peer_id, payload } => {
            if let Some(target) = names.iter().position(|n| n == peer_id) {
                if mesh.severed(from, target) {
                    return;
                }
                pending.push((
                    target,
                    p2p::Event::Message {
                        from: from_name.to_string(),
                        payload: payload.clone(),
                    },
                ));
            }
        }
        p2p::Command::PutHistory { lobby, chunk, data } => {
            history.put(lobby.clone(), *chunk, data.clone());
            for target in mesh.component(from) {
                if target == from {
                    continue;
                }
                pending.push((
                    target,
                    p2p::Event::HistoryChunk {
                        lobby: lobby.clone(),
                        chunk: *chunk,
                        data: data.clone(),
                    },
                ));
            }
        }
        p2p::Command::FetchHistory { lobby, chunk } => {
            if let Some(data) = history.fetch(lobby, *chunk) {
                pending.push((
                    from,
                    p2p::Event::HistoryChunk {
                        lobby: lobby.clone(),
                        chunk: *chunk,
                        data,
                    },
                ));
            }
        }
        p2p::Command::Publish { topic, data } => {
            for target in mesh.component(from) {
                if target == from {
                    continue;
                }
                pending.push((
                    target,
                    p2p::Event::Gossip {
                        topic: topic.clone(),
                        from: from_name.to_string(),
                        data: data.clone(),
                    },
                ));
            }
        }
        p2p::Command::FetchRoster { .. } => {
            for target in mesh.component(from) {
                if target == from || mesh.severed(from, target) {
                    continue;
                }
                pending.push((target, p2p::Event::PeerConnected(from_name.to_string())));
                if let Some(name) = names.get(target) {
                    pending.push((from, p2p::Event::PeerConnected(name.clone())));
                }
            }
        }
        p2p::Command::Dial { .. }
        | p2p::Command::DialForce { .. }
        | p2p::Command::ReserveRelay { .. }
        | p2p::Command::SetMdns { .. }
        | p2p::Command::AddKadPeer { .. }
        | p2p::Command::ProvideLobby { .. }
        | p2p::Command::FindLobby { .. }
        | p2p::Command::Subscribe { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::p2p;

    use super::route;
    use crate::clicker;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        for app in &mut mesh.apps {
            app.world_mut()
                .resource_mut::<p2p::Commands<clicker::CursorMsg>>()
                .push(p2p::Command::Subscribe {
                    topic: "clicker/alpha/pos".to_string(),
                });
        }
        route(&mut mesh);
        for app in &mut mesh.apps {
            assert!(
                app.world()
                    .resource::<p2p::Commands<clicker::CursorMsg>>()
                    .is_empty()
            );
        }
    }
}
