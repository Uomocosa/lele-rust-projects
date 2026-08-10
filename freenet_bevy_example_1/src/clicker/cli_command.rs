use bevy::prelude::Message;

use super::cli_command_help_text;
use super::cli_command_parse;

#[derive(Debug, Clone, Message)]
pub enum CliCommand {
    Increment,
    Status,
    Help,
    Quit,
}

#[rustfmt::skip]
impl CliCommand {
    pub fn parse(input: &str) -> Option<Self> { cli_command_parse::parse(input) }
    pub fn help_text() -> String { cli_command_help_text::help_text() }
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
