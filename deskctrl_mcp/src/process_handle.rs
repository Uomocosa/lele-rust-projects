use std::sync::{Arc, atomic::AtomicBool};

use tokio::sync::{Mutex, mpsc, oneshot};

use crate::OutputBuffer;

pub struct ProcessHandle {
    pub cmd: String,
    /// The operating system's pid, for correlating a spawned GUI app with its window
    /// (`list_windows` reports the same number). `None` if the child exited before we asked.
    pub os_pid: Option<u32>,
    pub output: Arc<Mutex<OutputBuffer>>,
    pub stdin_tx: Option<mpsc::Sender<String>>,
    pub alive: Arc<AtomicBool>,
    pub kill_tx: Option<oneshot::Sender<()>>,
}
