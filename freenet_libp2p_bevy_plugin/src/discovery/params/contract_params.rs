use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Deref)]
pub struct ContractParams(pub Vec<u8>);

#[cfg(test)]
mod tests {
    use super::ContractParams;

    #[test]
    fn test_usage() {
        let params = ContractParams(vec![1, 2]);
        assert_eq!(params.len(), 2);
        let bytes = bincode::serialize(&params).unwrap_or_default();
        let decoded: ContractParams = bincode::deserialize(&bytes).unwrap_or_default();
        assert_eq!(decoded, params);
    }
}
