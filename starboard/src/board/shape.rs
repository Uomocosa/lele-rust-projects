use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    Square,
    Circle,
    Star,
    Triangle,
    Hex,
    Heart,
}

#[cfg(test)]
mod tests {
    use super::Shape;

    #[test]
    fn test_usage() {
        let s = Shape::Star;
        assert_eq!(s, Shape::Star);
    }
}
