use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use crate::p2p::testing::Output;
use crate::p2p::testing::ProcessOrchestratorMethod;
use crate::p2p::testing::SpawnedPeer;

pub struct ProcessOrchestrator {
    pub(crate) exe: PathBuf,
    pub(crate) timeout: Duration,
    pub(crate) children: Vec<SpawnedPeer>,
    pub(crate) _tx: mpsc::Sender<String>,
    pub(crate) rx: mpsc::Receiver<String>,
}

#[rustfmt::skip]
impl ProcessOrchestrator {
    pub fn new(timeout: Duration) -> Result<Self, crate::p2p::testing::Error> { ProcessOrchestratorMethod::new(timeout) }
    pub fn spawn(&mut self, tag: &str) -> Result<(), crate::p2p::testing::Error> { ProcessOrchestratorMethod::spawn(self, tag) }
    pub fn collect(self) -> Output { ProcessOrchestratorMethod::collect(self) }
}
