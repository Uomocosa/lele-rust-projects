use crate::engine;

pub fn recv_engine(handle: &engine::EngineHandle) -> Option<engine::Snapshot> {
    let reply = handle.reply.lock().ok()?.recv().ok()?;
    match reply {
        engine::EngineReply::Snapshot(snapshot) => Some(snapshot),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::mpsc;

    use crate::engine;
    use crate::engine::Snapshot;

    use super::recv_engine;

    #[test]
    fn test_usage() {
        let (_cmd_tx, _cmd_rx) = mpsc::channel::<engine::EngineCmd>();
        let (reply_tx, reply_rx) = mpsc::channel::<engine::EngineReply>();
        reply_tx
            .send(engine::EngineReply::Snapshot(Snapshot {
                tick: 2,
                bodies: BTreeMap::new(),
            }))
            .unwrap();
        let handle = engine::EngineHandle {
            cmd: _cmd_tx.clone(),
            reply: std::sync::Mutex::new(reply_rx),
        };
        let snapshot = recv_engine(&handle);
        assert!(snapshot.is_some());
    }
}
