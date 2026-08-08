use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex, atomic::AtomicU32},
};

use tokio::sync::Mutex;

use crate::Server;

pub fn new(artifacts_dir: Option<String>) -> Server {
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").ok();
    let chat_id = std::env::var("TELEGRAM_CHAT_ID").ok();
    Server {
        processes: Arc::new(Mutex::new(HashMap::new())),
        next_id: Arc::new(AtomicU32::new(1)),
        artifacts_dir,
        bot_token,
        chat_id,
        action_log: Arc::new(StdMutex::new(Vec::new())),
        last_screenshot: Arc::new(StdMutex::new(None)),
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
