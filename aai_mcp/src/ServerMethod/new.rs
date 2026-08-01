use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicU32},
};

use tokio::sync::Mutex;

use crate::Server;

pub fn new(artifacts_dir: Option<String>) -> Server {
    Server {
        processes: Arc::new(Mutex::new(HashMap::new())),
        next_id: Arc::new(AtomicU32::new(1)),
        artifacts_dir,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use crate::Server;

    #[test]
    fn test_usage() {
        let server = Server::new();
        assert_eq!(server.next_id.load(Ordering::Relaxed), 1);
        assert!(server.processes.blocking_lock().is_empty());
    }
}
