use crate::engine;

#[derive(Debug)]
pub enum EngineReply {
    Snapshot(engine::Snapshot),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::EngineReply;
    use crate::engine::Snapshot;

    #[test]
    fn test_usage() {
        let snap = Snapshot {
            tick: 1,
            bodies: BTreeMap::new(),
        };
        let _ = EngineReply::Snapshot(snap);
    }
}
