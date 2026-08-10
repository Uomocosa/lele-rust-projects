use bevy::prelude::Message;

use crate::clicker::LogMessage;

#[derive(Message, Debug, Clone)]
pub struct LogMessageAdded(pub LogMessage);

#[cfg(test)]
mod tests {
    use super::LogMessageAdded;
    use crate::clicker::LogMessage;

    #[test]
    fn test_usage() {
        let msg = LogMessageAdded(LogMessage::new("hi"));
        assert_eq!(msg.0.text, "hi");
    }
}
