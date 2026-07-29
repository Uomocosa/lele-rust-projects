use bevy::prelude::Message;

use crate::clicker::cli::CliCommandMethod;

#[derive(Debug, Clone, Message)]
pub enum CliCommand {
    Increment,
    Status,
    Help,
    Quit,
}

impl CliCommand {
    #[rustfmt::skip]
    pub fn parse(input: &str) -> Option<Self> { CliCommandMethod::parse(input) }

    #[rustfmt::skip]
    pub fn help_text() -> String { CliCommandMethod::help_text() }
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
