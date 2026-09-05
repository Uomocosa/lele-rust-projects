use crate::board;

#[must_use]
pub fn tool_for(player: &[u8; 32], lobby: &str) -> board::Tool {
    let bytes = bincode::serialize(&(player, lobby)).unwrap_or_default();
    let hash = blake3::hash(&bytes);
    let b = hash.as_bytes();
    let shapes = [
        board::Shape::Square,
        board::Shape::Circle,
        board::Shape::Star,
        board::Shape::Triangle,
        board::Shape::Hex,
        board::Shape::Heart,
    ];
    let idx = usize::from(b[0] % 6);
    board::Tool {
        shape: shapes.get(idx).copied().unwrap_or(board::Shape::Square),
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
