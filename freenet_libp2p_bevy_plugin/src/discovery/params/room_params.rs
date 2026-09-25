use super::contract_params::ContractParams;
use super::room_name::RoomName;
use super::unique_game_id::UniqueGameId;

#[must_use]
pub fn room_params(id: &UniqueGameId, room: &RoomName) -> ContractParams {
    ContractParams(bincode::serialize(&((*id).clone(), (*room).clone())).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::room_params;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let id = discovery::params::UniqueGameId::new(
            &discovery::params::GameName("chess".to_string()),
            "token",
        );
        let alpha = room_params(&id, &discovery::params::RoomName("alpha".to_string()));
        let beta = room_params(&id, &discovery::params::RoomName("beta".to_string()));
        assert_ne!(alpha, beta);
        let decoded: (String, String) = bincode::deserialize(&alpha).unwrap_or_default();
        assert_eq!(decoded.0, "chess/token");
        assert_eq!(decoded.1, "alpha");
    }
}
