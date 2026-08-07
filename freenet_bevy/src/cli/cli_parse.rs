use super::cli::Cli;

pub fn parse() -> Cli {
    Cli {
        mode: super::cli_parse_mode::parse_mode(),
        freenet_role: super::cli_parse_role::parse_role(),
        node: super::cli_parse_node::parse_node(),
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
        let _ = cli.freenet_role;
        let _ = cli.node;
        let _ = cli.p2p_port;
    }
}
