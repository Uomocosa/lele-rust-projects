use std::time::Duration;

use crate::discovery;
use discovery::id::{RemotePeerId, UniqueGameId};
use discovery::link::{dialable, wait_ready};
use discovery::lobby_rooms::connect_retry;
use discovery::session::apply_command::apply_command;
use discovery::session::apply_tap::apply_tap;
use discovery::session::maintain::maintain;
use discovery::session::maintain_board::maintain_board;
use discovery::session::snapshot::snapshot;
use discovery::session::{RunConfig, Session};

pub async fn run(run: RunConfig) {
    let RunConfig {
        config,
        endpoint,
        mut link,
        mut tap,
        mut ready,
        mut commands,
        multiplayer,
        events,
    } = run;
    let Some((peer_id, addrs)) = wait_ready(&mut ready).await else {
        return;
    };
    let id = UniqueGameId::new(&config.game_name, &config.token);
    let mut session = Session::new(
        id.clone(),
        RemotePeerId(peer_id),
        dialable(addrs),
        config.capacity,
    );
    let mut index = connect_retry("127.0.0.1", *endpoint, &id).await;
    let timing = config.timing;
    let mut mesh_tick = tokio::time::interval(Duration::from_secs(timing.tick_secs.max(1)));
    let mut board_tick = tokio::time::interval(Duration::from_secs(timing.tick_secs.max(1)));
    loop {
        tokio::select! {
            Some(command) = commands.recv() => apply_command(&mut session, &link, command, &events),
            Some(event) = tap.recv() => apply_tap(&mut session, &link, event, &events),
            _ = mesh_tick.tick() => maintain(&mut session, &mut link, &timing),
            _ = board_tick.tick() => maintain_board(&mut session, &mut index, &timing, &events).await,
        }
        let _ = multiplayer.send(snapshot(&session));
    }
}

// no test_usage necessary
