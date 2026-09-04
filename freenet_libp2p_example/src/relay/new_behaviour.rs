use libp2p::StreamProtocol;
use libp2p::request_response::{self, Config, ProtocolSupport};

use crate::relay;

#[must_use]
pub fn new_behaviour() -> relay::Behaviour {
    request_response::Behaviour::<relay::LetterCodec>::new(
        [(StreamProtocol::new("/letters/1.0.0"), ProtocolSupport::Full)],
        Config::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::new_behaviour;

    #[test]
    fn test_usage() {
        let _ = new_behaviour();
    }
}
