use crate::error;
use crate::params;

pub fn decode_params(bytes: &[u8]) -> Result<params::Params, error::Error> {
    bincode::deserialize(bytes).map_err(|_| error::Error::InvalidParams)
}

#[cfg(test)]
mod tests {
    use crate::{error, params};

    use super::decode_params;

    #[test]
    fn test_usage() {
        let p = params::Params {
            namespace: [7; 32],
            max_members: 4,
        };
        let encoded = bincode::serialize(&p).unwrap();
        assert_eq!(decode_params(&encoded).unwrap(), p);
        assert_eq!(decode_params(b"junk"), Err(error::Error::InvalidParams));
    }
}
