use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use crate::discovery;

#[must_use]
pub fn stagger_due(
    now: Instant,
    connected: &HashMap<String, u32, std::hash::RandomState>,
    staggers: &mut HashMap<String, (VecDeque<String>, Instant), std::hash::RandomState>,
) -> Vec<(String, String)> {
    staggers.retain(|peer, _| !connected.contains_key(peer));
    let mut due = Vec::new();
    for (peer, (queue, at)) in staggers.iter_mut() {
        if *at > now {
            continue;
        }
        if let Some(addr) = queue.pop_front() {
            due.push((peer.clone(), addr));
            *at = now
                .checked_add(std::time::Duration::from_secs(discovery::STAGGER_SECS))
                .unwrap_or(now);
        }
    }
    staggers.retain(|_, (queue, _)| !queue.is_empty());
    due
}

#[cfg(test)]
mod tests {
    use super::stagger_due;
    use std::collections::{HashMap, VecDeque};
    use std::time::{Duration, Instant};

    fn queue(addrs: &[&str], at: Instant) -> (VecDeque<String>, Instant) {
        (addrs.iter().map(|a| (*a).to_string()).collect(), at)
    }

    #[test]
    fn test_usage() {
        let now = Instant::now();
        let connected: HashMap<String, u32> = HashMap::new();
        let mut staggers: HashMap<String, (VecDeque<String>, Instant)> = HashMap::new();
        staggers.insert("fresh".to_string(), queue(&["a", "b"], now));
        staggers.insert(
            "waiting".to_string(),
            queue(
                &["c"],
                now.checked_add(Duration::from_secs(60)).unwrap_or(now),
            ),
        );
        staggers.insert("linked".to_string(), queue(&["d"], now));
        let mut linked = HashMap::new();
        linked.insert("linked".to_string(), 1);
        let due = stagger_due(now, &linked, &mut staggers);
        assert_eq!(due, vec![("fresh".to_string(), "a".to_string())]);
        assert!(staggers.contains_key("fresh"));
        assert!(staggers.contains_key("waiting"));
        assert!(!staggers.contains_key("linked"));
        let later = now.checked_add(Duration::from_secs(3)).unwrap_or(now);
        let due = stagger_due(later, &connected, &mut staggers);
        assert_eq!(due, vec![("fresh".to_string(), "b".to_string())]);
        assert!(!staggers.contains_key("fresh"));
    }
}
