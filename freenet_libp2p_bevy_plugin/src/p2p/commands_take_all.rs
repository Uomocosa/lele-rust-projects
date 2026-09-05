use super::commands::Commands;
use crate::p2p;

pub fn take_all<T: p2p::Message>(commands: &mut Commands<T>) -> Vec<p2p::Command<T>> {
    std::mem::take(commands)
}

#[cfg(test)]
mod tests {
    use super::take_all;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut c = p2p::Commands::<()>::default();
        c.push(p2p::Command::Dial {
            peer_id: "p".to_string(),
            addrs: vec![],
        });
        assert_eq!(take_all(&mut c).len(), 1);
    }
}
