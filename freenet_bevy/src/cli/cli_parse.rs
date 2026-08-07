use super::cli::Cli;

pub fn parse() -> Cli {
    Cli {
        mode: super::cli_parse_mode::parse_mode(),
        role: super::cli_parse_role::parse_role(),
        p2p_port: super::cli_parse_p2p_port::parse_p2p_port(),
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_usage() {
        let cli = parse();
        let _ = cli.mode;
        let _ = cli.role;
        let _ = cli.p2p_port;
    }
}
