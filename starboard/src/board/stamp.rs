use serde::{Deserialize, Serialize};

use super::stamp_new;
use crate::board;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stamp {
    pub author: [u8; 32],
    pub seq: u64,
    pub pos: [f32; 2],
    pub tool: board::Tool,
}

#[rustfmt::skip]
impl Stamp {
    #[must_use]
    pub const fn new(author: [u8; 32], seq: u64, pos: [f32; 2], tool: board::Tool) -> Self { stamp_new::new(author, seq, pos, tool) }
}

#[cfg(test)]
mod tests {
    use super::Stamp;
    use crate::board;

    #[test]
    fn test_usage() {
        let s = Stamp::new(
            [0; 32],
            1,
            [0.5, 0.5],
            board::Tool {
                shape: board::Shape::Circle,
                color: [255, 0, 0],
            },
        );
        assert_eq!(s.seq, 1);
    }
}
