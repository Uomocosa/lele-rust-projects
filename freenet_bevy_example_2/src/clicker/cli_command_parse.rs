use crate::clicker;

pub fn parse(input: &str) -> Option<clicker::CliCommand> {
    match input.trim().to_lowercase().as_str() {
        "increment" | "inc" | "+" => Some(clicker::CliCommand::Increment),
        "status" | "s" => Some(clicker::CliCommand::Status),
        "quit" | "q" | "exit" => Some(clicker::CliCommand::Quit),
        "help" | "h" => Some(clicker::CliCommand::Help),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::clicker;

    #[test]
    fn test_usage() {
        assert!(matches!(
            parse("increment"),
            Some(clicker::CliCommand::Increment)
        ));
        assert!(matches!(parse("inc"), Some(clicker::CliCommand::Increment)));
        assert!(matches!(parse("+"), Some(clicker::CliCommand::Increment)));
        assert!(matches!(parse("status"), Some(clicker::CliCommand::Status)));
        assert!(matches!(parse("quit"), Some(clicker::CliCommand::Quit)));
        assert!(matches!(parse("help"), Some(clicker::CliCommand::Help)));
        assert!(parse("unknown").is_none());
    }
}
