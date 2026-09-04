use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

#[cfg(test)]
mod tests {
    use super::Payload;

    #[test]
    fn test_usage() {
        let payload = Payload {
            x: 1.0,
            y: 2.0,
            vx: 0.0,
            vy: 0.0,
        };
        let encoded = bincode::serialize(&payload);
        let decoded = encoded.ok().and_then(|e| bincode::deserialize(&e).ok());
        assert_eq!(decoded, Some(payload));
    }
}
