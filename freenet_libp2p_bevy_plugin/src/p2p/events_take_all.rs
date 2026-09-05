use super::events::Events;
use crate::p2p;

pub fn take_all<T: p2p::Message>(events: &mut Events<T>) -> Vec<p2p::Event<T>> {
    std::mem::take(events)
}

#[cfg(test)]
mod tests {
    use super::take_all;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut e = p2p::Events::<()>::default();
        e.push(p2p::Event::Error("oops".to_string()));
        assert_eq!(take_all(&mut e).len(), 1);
    }
}
