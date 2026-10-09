use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;

use crate::discovery;
use crate::net_id;
use crate::p2p;
use discovery::session::{Input, Output, Session};

pub async fn run(
    config: discovery::Config,
    endpoint: discovery::FreenetEndpoint,
    channels: discovery::Channels,
) {
    let discovery::Channels {
        mut net,
        mut commands,
        snapshots,
        events,
    } = channels;
    let Some(ready) = discovery::libp2p::wait_ready(&mut net.ready).await else {
        return;
    };
    let me = net_id::Peer {
        addrs: discovery::libp2p::dialable(ready.addrs),
        id: ready.id,
    };
    let timing = config.timing;
    let capacity = config.capacity;
    let mut session = Session::new(me.clone(), timing);
    let (target_tx, target_rx) = tokio::sync::watch::channel(None);
    let (lobby_tx, mut lobbies) = tokio::sync::mpsc::unbounded_channel();
    let params = discovery::freenet::contract_params(&config.game_name, &config.token);
    tokio::spawn(async move {
        let client =
            discovery::freenet::connect_retry("127.0.0.1", *endpoint, &params, &target_rx).await;
        discovery::freenet::run_lobby(client, me.id, capacity, timing, target_rx, lobby_tx).await;
    });
    let mut tick = tokio::time::interval(Duration::from_secs(timing.tick_secs.max(1)));
    loop {
        let input = tokio::select! {
            Some(command) = commands.recv() => Input::Command(command),
            Some(event) = net.events.recv() => Input::Net(event),
            Some(lobby) = lobbies.recv() => Input::Lobby(lobby),
            _ = tick.tick() => Input::Tick,
        };
        adopt_observed(&mut session, &mut net.observed);
        discovery::session::handle(&mut session, input, discovery::now_epoch());
        flush(&mut session, &net.commands, &events);
        let target = discovery::session::publish_target(&session);
        target_tx.send_if_modified(|current| {
            let changed = *current != target;
            if changed {
                current.clone_from(&target);
            }
            changed
        });
        let _ = snapshots.send(discovery::session::snapshot(&session));
    }
}

// needed helper: adopts libp2p-observed addresses so peers can dial us back
fn adopt_observed(session: &mut Session, observed: &mut Receiver<Option<Vec<net_id::PeerAddr>>>) {
    if !observed.has_changed().unwrap_or(false) {
        return;
    }
    let latest = observed.borrow_and_update().clone();
    if let Some(addrs) = latest {
        session.me.addrs = discovery::libp2p::dialable(addrs);
    }
}

// needed helper: performs the session's queued effects
fn flush(
    session: &mut Session,
    net: &UnboundedSender<p2p::NetCommand>,
    events: &UnboundedSender<discovery::Event>,
) {
    for output in std::mem::take(&mut session.outputs) {
        match output {
            Output::Event(event) => {
                let _ = events.send(event);
            }
            Output::Net(command) => {
                let _ = net.send(command);
            }
        }
    }
}

// no test_usage necessary
