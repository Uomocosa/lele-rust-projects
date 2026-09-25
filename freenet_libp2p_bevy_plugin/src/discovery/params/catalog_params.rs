use super::super::constants;
use super::contract_params::ContractParams;
use super::unique_game_id::UniqueGameId;

#[must_use]
pub fn catalog_params(id: &UniqueGameId) -> ContractParams {
    ContractParams(
        bincode::serialize(&((*id).clone(), constants::DIRECTORY_LOBBY.to_string()))
            .unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::catalog_params;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let id = discovery::params::UniqueGameId::new(
            &discovery::params::GameName("chess".to_string()),
            "token",
        );
        let dir = catalog_params(&id);
        let decoded: (String, String) = bincode::deserialize(&dir).unwrap_or_default();
        assert_eq!(decoded.0, "chess/token");
        assert_eq!(decoded.1, discovery::DIRECTORY_LOBBY);
        assert_ne!(
            catalog_params(&id),
            catalog_params(&discovery::params::UniqueGameId::new(
                &discovery::params::GameName("other".to_string()),
                "token",
            ))
        );
    }
}
