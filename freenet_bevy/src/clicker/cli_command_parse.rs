use crate::clicker::cli_command::CliCommand;

pub fn parse(input: &str) -> Option<CliCommand> {
    match input.trim().to_lowercase().as_str() {
        "increment" | "inc" | "+" => Some(CliCommand::Increment),
        "status" | "s" => Some(CliCommand::Status),
        "quit" | "q" | "exit" => Some(CliCommand::Quit),
        "help" | "h" => Some(CliCommand::Help),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::clicker::cli_command::CliCommand;

    #[test]
    fn test_usage() {
        assert!(matches!(parse("increment"), Some(CliCommand::Increment)));
        assert!(matches!(parse("inc"), Some(CliCommand::Increment)));
        assert!(matches!(parse("+"), Some(CliCommand::Increment)));
        assert!(matches!(parse("status"), Some(CliCommand::Status)));
        assert!(matches!(parse("quit"), Some(CliCommand::Quit)));
        assert!(matches!(parse("help"), Some(CliCommand::Help)));
        assert!(parse("unknown").is_none());
    }
}
