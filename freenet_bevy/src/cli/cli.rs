use crate::freenet;

use super::cli_parse;
use super::cli_parse_mode;
use super::cli_parse_p2p_port;
use super::cli_parse_role;

pub struct Cli {
    pub mode: super::Mode,
    pub role: freenet::FreenetRole,
    pub p2p_port: u16,
    pub has_role: bool,
}

#[rustfmt::skip]
impl Cli {
    pub fn parse() -> Self {
        cli_parse::parse()
    }
    pub fn parse_mode() -> super::Mode {
        cli_parse_mode::parse_mode()
    }
    pub fn parse_role() -> freenet::FreenetRole {
        cli_parse_role::parse_role()
    }
    pub fn parse_p2p_port() -> u16 {
        cli_parse_p2p_port::parse_p2p_port()
    }
}

// no test_usage necessary
