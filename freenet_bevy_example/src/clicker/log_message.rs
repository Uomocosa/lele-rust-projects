#[derive(Debug, Clone)]
pub struct LogMessage {
    pub timestamp: String,
    pub text: String,
}

impl LogMessage {
    pub fn new(text: impl Into<String>) -> Self {
        let now = chrono::Local::now();
        Self {
            timestamp: now.format("%H:%M:%S").to_string(),
            text: text.into(),
        }
    }

    pub fn display(&self) -> String {
        format!("[{}] {}", self.timestamp, self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::LogMessage;

    #[test]
    fn test_usage() {
        let msg = LogMessage::new("connected to freenet");
        assert!(msg.display().contains("connected to freenet"));
    }
}
