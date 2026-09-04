use serde::{Deserialize, Serialize};

use crate::board::Tool;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stamp {
    pub author: [u8; 32],
    pub seq: u64,
    pub pos: [f32; 2],
    pub tool: Tool,
}

#[cfg(test)]
mod tests {
    use super::Stamp;
    use crate::board::{Shape, Tool};

    #[test]
    fn test_usage() {
        let s = Stamp {
            author: [0; 32],
            seq: 1,
            pos: [0.5, 0.5],
            tool: Tool {
                shape: Shape::Circle,
                color: [255, 0, 0],
            },
        };
        assert_eq!(s.seq, 1);
    }
}
