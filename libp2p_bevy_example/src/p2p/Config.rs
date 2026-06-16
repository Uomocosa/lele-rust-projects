use crate::p2p::ConfigMethod;

#[derive(Clone, Debug)]
pub struct Config {
    pub enable_mdns: bool,
    pub enable_manual_dial: bool,
    pub heartbeat_interval_ms: u64,
    pub connection_timeout_ms: u64,
    pub auto_accept_join: bool,
    pub max_players: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_mdns: true,
            enable_manual_dial: true,
            heartbeat_interval_ms: 5000,
            connection_timeout_ms: 30000,
            auto_accept_join: true,
            max_players: None,
        }
    }
}

#[rustfmt::skip]
impl Config {
    pub fn new() -> Self { ConfigMethod::new() }
    pub fn coop() -> Self { ConfigMethod::coop() }
    pub fn pvp() -> Self { ConfigMethod::pvp() }
    pub fn lan_coop() -> Self { ConfigMethod::lan_coop() }
    pub fn lan_pvp() -> Self { ConfigMethod::lan_pvp() }
    pub fn mmo() -> Self { ConfigMethod::mmo() }
    pub fn with_auto_accept(self, accept: bool) -> Self { ConfigMethod::with_auto_accept(self, accept) }
    pub fn with_connection_timeout(self, ms: u64) -> Self { ConfigMethod::with_connection_timeout(self, ms) }
    pub fn with_heartbeat(self, ms: u64) -> Self { ConfigMethod::with_heartbeat(self, ms) }
    pub fn with_manual_dial(self, enabled: bool) -> Self { ConfigMethod::with_manual_dial(self, enabled) }
    pub fn with_max_players(self, max: usize) -> Self { ConfigMethod::with_max_players(self, max) }
    pub fn with_mdns(self, enabled: bool) -> Self { ConfigMethod::with_mdns(self, enabled) }
}
