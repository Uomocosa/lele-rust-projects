use crate::engine;

pub fn send_cmd(handle: &engine::EngineHandle, cmd: engine::EngineCmd) {
    let _ = handle.cmd.send(cmd);
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use crate::engine;

    use super::send_cmd;

    #[test]
    fn test_usage() {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (_reply_tx, reply_rx) = mpsc::channel();
        let handle = engine::EngineHandle {
            cmd: cmd_tx,
            reply: std::sync::Mutex::new(reply_rx),
        };
        send_cmd(&handle, engine::EngineCmd::Spawn([1; 32]));
        assert!(cmd_rx.recv().is_ok());
    }
}
