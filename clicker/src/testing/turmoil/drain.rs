use freenet_libp2p_bevy_plugin::p2p;
use futures_util::future::LocalBoxFuture;

use super::prune::prune;
use super::up_link::UpLink;
use super::write_event::write_event;
use crate::clicker;

pub fn drain(link: &mut UpLink) -> LocalBoxFuture<'_, ()> {
    Box::pin(async move {
        let mut pending = Vec::new();
        while let Ok((origin, id, event)) = link.inbox_rx.try_recv() {
            if let p2p::Event::PeerDisconnected(peer) = &event {
                prune(&mut link.app, peer, &mut link.dead);
            }
            if matches!(event, p2p::Event::Gossip { .. }) {
                if !link.seen.insert((origin.clone(), id)) {
                    continue;
                }
                let envelope = (origin, id, event.clone());
                for (target, stream) in &mut link.outbound {
                    if link.dead.contains(target) {
                        continue;
                    }
                    write_event(stream, &envelope).await.ok();
                }
            }
            pending.push(event);
        }
        if !pending.is_empty() {
            link.app
                .world_mut()
                .resource_mut::<p2p::Events<clicker::CursorMsg>>()
                .extend(pending);
        }
    })
}

// no test_usage necessary
