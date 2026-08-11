use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub player_id: u64,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub tick: u64,
    pub sent_at_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::Snapshot;

    #[test]
    fn test_usage() {
        let snapshot = Snapshot {
            player_id: 1,
            x: 10.0,
            y: 20.0,
            vx: 1.0,
            vy: 0.0,
            tick: 5,
            sent_at_ms: 100,
        };
        let encoded = bincode::serialize(&snapshot);
        let decoded = encoded.ok().and_then(|e| bincode::deserialize(&e).ok());
        assert_eq!(decoded, Some(snapshot));
    }
}
