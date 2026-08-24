use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};

use bevy::prelude::Resource;

use super::engine_handle_recv_engine;
use super::engine_handle_send_cmd;
use crate::engine;

/// Resource giving the render `App` a handle to the single deterministic sim worker thread. The
/// worker owns the headless `engine::Engine` (an `App`, which is `!Send + !Sync`, so it cannot
/// live inside a Bevy resource) and steps it on demand, returning snapshots over a channel.
#[derive(Resource)]
pub struct EngineHandle {
    pub cmd: Sender<engine::EngineCmd>,
    pub reply: Mutex<Receiver<engine::EngineReply>>,
}

#[rustfmt::skip]
impl EngineHandle {
    pub fn send_cmd(&self, cmd: engine::EngineCmd) { engine_handle_send_cmd::send_cmd(self, cmd) }
    pub fn recv_engine(&self) -> Option<engine::Snapshot> { engine_handle_recv_engine::recv_engine(self) }
}
// no test_usage necessary - thin delegates, exercised by spawn_engine and the boxes tests
