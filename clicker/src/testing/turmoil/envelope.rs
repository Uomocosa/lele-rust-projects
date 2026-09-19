use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;

pub type Envelope = (String, u64, p2p::Event<clicker::CursorMsg>);

// no test_usage necessary
