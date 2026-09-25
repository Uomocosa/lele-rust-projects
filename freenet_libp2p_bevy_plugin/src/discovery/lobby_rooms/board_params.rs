use crate::discovery;
use discovery::id::UniqueGameId;

#[must_use]
pub fn board_params(id: &UniqueGameId) -> Vec<u8> {
    id.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::board_params;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let id = discovery::id::UniqueGameId::new(
            &discovery::id::GameName("chess".to_string()),
            &discovery::id::GameToken("token".to_string()),
        );
        assert_eq!(board_params(&id), b"chess/token".to_vec());
    }
}
