use bevy::prelude::Message;

#[derive(Message, Debug, Clone)]
pub struct ConnectionChanged {
    pub connected: bool,
}

#[cfg(test)]
mod tests {
    use super::ConnectionChanged;

    #[test]
    fn test_usage() {
        let msg = ConnectionChanged { connected: true };
        assert!(msg.connected);
    }
}
