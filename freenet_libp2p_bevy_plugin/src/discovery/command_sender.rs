use bevy::prelude::Resource;
use derive_more::Deref;

use super::Command;

#[derive(Resource, Debug, Deref)]
pub struct CommandSender(pub tokio::sync::mpsc::UnboundedSender<Command>);

#[cfg(test)]
mod tests {
    use super::CommandSender;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = CommandSender(tx);
        assert!(sender.send(discovery::Command::Leave).is_ok());
    }
}
