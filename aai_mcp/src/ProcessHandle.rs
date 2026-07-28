use std::sync::{Arc, atomic::AtomicBool};

use tokio::sync::{Mutex, mpsc, oneshot};

pub struct ProcessHandle {
    pub cmd: String,
    pub output_buf: Arc<Mutex<String>>,
    pub stdin_tx: Option<mpsc::Sender<String>>,
    pub alive: Arc<AtomicBool>,
    pub kill_tx: Option<oneshot::Sender<()>>,
}
