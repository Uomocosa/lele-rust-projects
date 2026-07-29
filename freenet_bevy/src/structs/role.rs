use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Publish,
    Subscribe,
}

#[cfg(test)]
mod tests {
    use super::Role;

    #[test]
    fn test_usage() {
        assert_ne!(Role::Publish, Role::Subscribe);
    }
}
