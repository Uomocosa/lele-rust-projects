use std::time::Duration;

use freenet_libp2p_bevy_plugin::{net_id, p2p};
use futures_util::future::LocalBoxFuture;

use super::drain::drain;
use super::flush::flush;
use super::redial_missing::redial_missing;
use super::up_link::UpLink;
use crate::clicker;

pub fn pump<'a>(link: &'a mut UpLink, name: &'a str) -> LocalBoxFuture<'a, ()> {
    Box::pin(async move {
        while let Ok((origin, stream)) = link.accept_rx.try_recv() {
            let known = link.outbound.contains_key(&origin);
            if !link.dead.contains(&origin) && !known {
                let inbox = link.inbox_tx.clone();
                super::register::register(
                    &mut link.app,
                    &mut link.outbound,
                    &inbox,
                    &origin,
                    stream,
                );
            }
        }
        link.app.update();
        let now = turmoil::elapsed();
        let own = *link.app.world().resource::<net_id::NetworkId>();
        redial_missing(link, name, now);
        let peers: Vec<String> = link.outbound.keys().cloned().collect();
        for peer in peers {
            if link.dead.contains(&peer) {
                continue;
            }
            let last = link.last_sync.get(&peer).copied().unwrap_or(Duration::ZERO);
            if now.saturating_sub(last) >= link.link_down_after {
                link.last_sync.insert(peer.clone(), now);
                link.app
                    .world_mut()
                    .resource_mut::<p2p::Commands<clicker::CursorMsg>>()
                    .push(p2p::Command::Send {
                        peer_id: peer,
                        payload: clicker::CursorMsg::SyncReq { requester: own },
                    });
            }
        }
        flush(link, name).await;
        drain(link).await;
    })
}

// no test_usage necessary
