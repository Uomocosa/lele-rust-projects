use std::process::Child;
use std::thread::JoinHandle;

pub(crate) struct SpawnedPeer {
    pub(crate) child: Option<Child>,
    pub(crate) handles: Vec<JoinHandle<()>>,
}
