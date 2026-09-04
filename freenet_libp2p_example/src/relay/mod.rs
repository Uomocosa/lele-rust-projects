// Discovery is via Freenet contract PeerRecord only — no mDNS/Kademlia
// mdns must be off — local test must use real Freenet discovery (no compile_error guard to avoid unexpected_cfgs lint)

pub mod behaviour;
pub mod drive_swarm;
pub mod gossip_state;
mod gossip_state_insert;
mod gossip_state_new;
mod gossip_state_should_accept;
pub mod letter_codec;
pub mod letter_request;
pub mod letter_response;
pub mod new_behaviour;
pub use behaviour::Behaviour;
pub use gossip_state::GossipState;
pub use letter_codec::LetterCodec;
pub use letter_request::LetterRequest;
pub use letter_response::LetterResponse;
