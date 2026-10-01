use std::time::Duration;

use crate::discovery;
use discovery::id::{RemotePeerId, UniqueGameId};
use discovery::link::{dialable, wait_ready};
use discovery::lobby_rooms::connect_retry;
use discovery::session::apply_command::apply_command;
use discovery::session::apply_tap::apply_tap;
use discovery::session::maintain::maintain;
use discovery::session::{RunConfig, Session, absorb_board, board_target, run_board, snapshot};

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
    let timing = config.timing;
    let (target_tx, target_rx) = tokio::sync::watch::channel(None);
    let (boards_tx, mut boards) = tokio::sync::mpsc::unbounded_channel();
    let me = session.me.clone();
    tokio::spawn(async move {
        let index = connect_retry("127.0.0.1", *endpoint, &id).await;
        run_board(index, me, config.capacity, timing, target_rx, boards_tx).await;
    });
    let mut mesh_tick = tokio::time::interval(Duration::from_secs(timing.tick_secs.max(1)));
    loop {
        tokio::select! {
            Some(command) = commands.recv() => {
                apply_command(&mut session, &link, command, &timing, &events);
            }
            Some(event) = tap.recv() => apply_tap(&mut session, &link, event, &timing, &events),
            Some(board) = boards.recv() => absorb_board(&mut session, board, &events),
            _ = mesh_tick.tick() => maintain(&mut session, &mut link, &timing, &events),
        }
        let target = board_target(&session);
        target_tx.send_if_modified(|current| {
            let changed = *current != target;
            if changed {
                current.clone_from(&target);
            }
            changed
        });
        let _ = multiplayer.send(snapshot(&session));
    }
}

// no test_usage necessary
