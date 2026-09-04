use libp2p::request_response;

use crate::relay;

pub type Behaviour = request_response::Behaviour<relay::LetterCodec>;

// no test_usage necessary
