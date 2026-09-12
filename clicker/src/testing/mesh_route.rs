use freenet_libp2p_bevy_plugin::p2p;

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
    let from_names = mesh.names.clone();
    let mut pending: Vec<(usize, p2p::Event<clicker::CursorMsg>)> = Vec::new();
    for (from, cmds) in from_names.iter().zip(out.iter()) {
        for cmd in cmds {
            route_command(&mesh.names, from, cmd, &mut pending);
        }
    }
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
    names: &[String; 3],
    from: &str,
    cmd: &p2p::Command<clicker::CursorMsg>,
    pending: &mut Vec<(usize, p2p::Event<clicker::CursorMsg>)>,
) {
    match cmd {
        p2p::Command::Send { peer_id, payload } => {
            if let Some(target) = names.iter().position(|n| n == peer_id) {
                pending.push((
                    target,
                    p2p::Event::Message {
                        from: from.to_string(),
                        payload: payload.clone(),
                    },
                ));
            }
        }
        p2p::Command::PutHistory { lobby, chunk, data } => {
            for (target, name) in names.iter().enumerate() {
                if name.as_str() == from {
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
        p2p::Command::Publish { topic, data } => {
            for (target, name) in names.iter().enumerate() {
                if name.as_str() == from {
                    continue;
                }
                pending.push((
                    target,
                    p2p::Event::Gossip {
                        topic: topic.clone(),
                        from: from.to_string(),
                        data: data.clone(),
                    },
                ));
            }
        }
        p2p::Command::Dial { .. }
        | p2p::Command::FetchHistory { .. }
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
        let mut mesh = testing::Mesh::three();
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
