use bevy::prelude::Message;

use crate::methods::clicker::cli::cli_command as cmd_method;

#[derive(Debug, Clone, Message)]
pub enum CliCommand {
    Increment,
    Status,
    Help,
    Quit,
}

#[rustfmt::skip]
impl CliCommand {
    pub fn parse(input: &str) -> Option<Self> { cmd_method::parse(input) }
    pub fn help_text() -> String { cmd_method::help_text() }
}

#[cfg(test)]
mod tests {
    use super::CliCommand;

    #[test]
    fn test_usage() {
        let cmd = CliCommand::parse("increment").unwrap();
        match cmd {
            CliCommand::Increment => {}
            _ => panic!("expected Increment"),
        }
        let help = CliCommand::help_text();
        assert!(help.contains("increment"));
        assert!(help.contains("status"));
    }
}
