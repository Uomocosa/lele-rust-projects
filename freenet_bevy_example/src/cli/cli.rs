use super::cli_parse;
use super::cli_parse_mode;
use super::cli_parse_p2p_port;

pub struct Cli {
    pub mode: super::Mode,
    pub p2p_port: u16,
}

#[rustfmt::skip]
impl Cli {
    pub fn parse() -> Self {
        cli_parse::parse()
    }
    pub fn parse_mode() -> super::Mode {
        cli_parse_mode::parse_mode()
    }
    pub fn parse_p2p_port() -> u16 {
        cli_parse_p2p_port::parse_p2p_port()
    }
}

// no test_usage necessary
