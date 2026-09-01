use crate::engine;

pub fn recv_reply(handle: &engine::EngineHandle) -> Option<engine::EngineReply> {
    handle.reply.lock().ok()?.recv().ok()
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use crate::engine;

    use super::recv_reply;

    #[test]
    fn test_usage() {
        let (_cmd_tx, _cmd_rx) = mpsc::channel::<engine::EngineCmd>();
        let (reply_tx, reply_rx) = mpsc::channel::<engine::EngineReply>();
        reply_tx
            .send(engine::EngineReply::Snapshot(engine::Snapshot::default()))
            .unwrap();
        let handle = engine::EngineHandle {
            cmd: _cmd_tx.clone(),
            reply: std::sync::Mutex::new(reply_rx),
        };
        assert!(recv_reply(&handle).is_some());
    }
}
