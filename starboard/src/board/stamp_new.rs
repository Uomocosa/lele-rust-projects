use super::stamp::Stamp;
use crate::board;

#[must_use]
pub const fn new(author: [u8; 32], seq: u64, pos: [f32; 2], tool: board::Tool) -> Stamp {
    Stamp {
        author,
        seq,
        pos,
        tool,
    }
}

#[cfg(test)]
mod tests {
    use super::new;
    use crate::board;

    #[test]
    fn test_usage() {
        let s = new(
            [2; 32],
            3,
            [0.0, 1.0],
            board::Tool {
                shape: board::Shape::Star,
                color: [0, 255, 0],
            },
        );
        assert_eq!(s.seq, 3);
    }
}
