use serde::{Deserialize, Serialize};

use super::tool_tool_for;
use crate::board;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    pub shape: board::Shape,
    pub color: [u8; 3],
}

#[rustfmt::skip]
impl Tool {
    #[must_use]
    pub fn tool_for(player: &[u8; 32], lobby: &str) -> Self { tool_tool_for::tool_for(player, lobby) }
}

#[cfg(test)]
mod tests {
    use super::Tool;
    use crate::board;

    #[test]
    fn test_usage() {
        let pid = [1u8; 32];
        let a = Tool::tool_for(&pid, "lobby-a");
        let b = Tool::tool_for(&pid, "lobby-a");
        assert_eq!(a, b);
        assert_eq!(a.shape, b.shape);
        let _ = board::Shape::Square;
    }
}
