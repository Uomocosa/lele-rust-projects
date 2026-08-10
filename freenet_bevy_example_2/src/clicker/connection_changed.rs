use bevy::prelude::Message;

use crate::clicker::ConnectionStatus;

#[derive(Message, Debug, Clone)]
pub struct ConnectionChanged {
    pub status: ConnectionStatus,
}

#[cfg(test)]
mod tests {
    use super::ConnectionChanged;
    use crate::clicker::ConnectionStatus;

    #[test]
    fn test_usage() {
        let msg = ConnectionChanged {
            status: ConnectionStatus::Connected,
        };
        assert_eq!(msg.status, ConnectionStatus::Connected);
    }
}
