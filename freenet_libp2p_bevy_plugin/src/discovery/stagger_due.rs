use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use super::constants;

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
                .checked_add(std::time::Duration::from_secs(constants::STAGGER_SECS))
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

    #[test]
    fn test_usage() {
        let now = Instant::now();
        let mut staggers: HashMap<String, (VecDeque<String>, Instant)> = HashMap::new();
        staggers.insert(
            "fresh".to_string(),
            (VecDeque::from(["a".to_string(), "b".to_string()]), now),
        );
        let due = stagger_due(now, &HashMap::new(), &mut staggers);
        assert_eq!(due, vec![("fresh".to_string(), "a".to_string())]);
        let later = now.checked_add(Duration::from_secs(3)).unwrap_or(now);
        let due = stagger_due(later, &HashMap::new(), &mut staggers);
        assert_eq!(due, vec![("fresh".to_string(), "b".to_string())]);
    }
}
