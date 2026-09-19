use freenet_libp2p_bevy_plugin::p2p;
use futures_util::future::LocalBoxFuture;

use super::prune::prune;
use super::up_link::UpLink;
use super::write_event::write_event;
use crate::clicker;

pub fn flush<'a>(link: &'a mut UpLink, name: &'a str) -> LocalBoxFuture<'a, ()> {
    Box::pin(async move {
        let cmds = link
            .app
            .world_mut()
            .resource_mut::<p2p::Commands<clicker::CursorMsg>>()
            .take_all();
        for cmd in cmds {
            let (targets, event) = match cmd {
                p2p::Command::Send { peer_id, payload } => (
                    vec![peer_id],
                    p2p::Event::Message {
                        from: name.to_string(),
                        payload,
                    },
                ),
                p2p::Command::PutHistory { lobby, chunk, data } => (
                    link.outbound.keys().cloned().collect(),
                    p2p::Event::HistoryChunk { lobby, chunk, data },
                ),
                p2p::Command::Publish { topic, data } => (
                    link.outbound.keys().cloned().collect(),
                    p2p::Event::Gossip {
                        topic,
                        from: name.to_string(),
                        data,
                    },
                ),
                _ => continue,
            };
            let id = link.seq;
            link.seq = link.seq.saturating_add(1);
            let envelope = (name.to_string(), id, event);
            write_targets(link, &targets, &envelope).await;
        }
    })
}

// needed helper: writes one envelope to each live target, pruning on failure
fn write_targets<'a>(
    link: &'a mut UpLink,
    targets: &'a [String],
    envelope: &'a (String, u64, p2p::Event<clicker::CursorMsg>),
) -> LocalBoxFuture<'a, ()> {
    Box::pin(async move {
        for target in targets {
            if link.dead.contains(target) {
                continue;
            }
            if let Some(stream) = link.outbound.get_mut(target)
                && write_event(stream, envelope).await.is_err()
            {
                prune(&mut link.app, target, &mut link.dead);
            }
        }
    })
}

// no test_usage necessary
