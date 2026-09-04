use serde::{Deserialize, Serialize};

use crate::board::Shape;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    pub shape: Shape,
    pub color: [u8; 3],
}

pub fn tool_for(player: &[u8; 32], lobby: &str) -> Tool {
    let bytes = bincode::serialize(&(player, lobby)).unwrap();
    let hash = blake3::hash(&bytes);
    let b = hash.as_bytes();
    let shapes = [
        Shape::Square,
        Shape::Circle,
        Shape::Star,
        Shape::Triangle,
        Shape::Hex,
        Shape::Heart,
    ];
    Tool {
        shape: shapes[(b[0] % 6) as usize],
        color: [b[1], b[2], b[3]],
    }
}

#[cfg(test)]
mod tests {
    use super::tool_for;

    #[test]
    fn test_usage() {
        let pid = [1u8; 32];
        let a = tool_for(&pid, "lobby-a");
        let b = tool_for(&pid, "lobby-a");
        assert_eq!(a, b);
        let c = tool_for(&pid, "lobby-b");
        assert_ne!(a.color, c.color);
    }
}
