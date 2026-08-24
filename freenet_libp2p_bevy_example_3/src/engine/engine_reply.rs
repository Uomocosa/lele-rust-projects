use crate::engine;

/// Replies returned by the sim worker. `Snapshot` corresponds to a `Step` of the plain
/// authoritative engine; `Predicted` is the rollback session's predicted state for the local box.
#[derive(Debug)]
pub enum EngineReply {
    Snapshot(engine::Snapshot),
    Predicted(engine::Snapshot),
}

#[cfg(test)]
mod tests {
    use super::EngineReply;
    use crate::engine;

    #[test]
    fn test_usage() {
        let _ = EngineReply::Snapshot(engine::Snapshot::default());
        let _ = EngineReply::Predicted(engine::Snapshot::default());
    }
}
