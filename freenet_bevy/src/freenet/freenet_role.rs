use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FreenetRole {
    Publish,
    Subscribe,
}

#[cfg(test)]
mod tests {
    use super::FreenetRole;

    #[test]
    fn test_usage() {
        assert_ne!(FreenetRole::Publish, FreenetRole::Subscribe);
    }
}
