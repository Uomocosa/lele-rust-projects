use crate::p2p;

pub fn take_all(commands: &mut p2p::Outbox) -> Vec<p2p::NetCommand> {
    std::mem::take(&mut commands.0)
}

#[cfg(test)]
mod tests {
    use super::take_all;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut commands = p2p::Outbox::default();
        commands.push(p2p::NetCommand::FindLobby {
            lobby: "lobby".to_string(),
        });
        assert_eq!(take_all(&mut commands).len(), 1);
        assert!(commands.is_empty());
    }
}
