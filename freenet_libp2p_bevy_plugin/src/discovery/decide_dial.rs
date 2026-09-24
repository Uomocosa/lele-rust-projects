use super::constants;
use super::dial_decision::DialDecision;

#[must_use]
pub fn decide_dial(
    own_peer_id: &str,
    peer_id: &str,
    first_seen_secs_ago: Option<u64>,
) -> DialDecision {
    let retry = first_seen_secs_ago.is_some();
    if own_peer_id >= peer_id {
        match first_seen_secs_ago {
            None => return DialDecision::Wait,
            Some(age) if age < constants::REDIAL_SECS => return DialDecision::Wait,
            _ => {}
        }
    }
    if retry {
        DialDecision::ForceDial
    } else {
        DialDecision::Dial
    }
}

#[cfg(test)]
mod tests {
    use super::decide_dial;
    use crate::discovery;

    #[test]
    fn test_usage() {
        assert_eq!(decide_dial("a", "b", None), discovery::DialDecision::Dial);
        assert_eq!(decide_dial("b", "a", None), discovery::DialDecision::Wait);
        assert_eq!(
            decide_dial("b", "a", Some(discovery::REDIAL_SECS)),
            discovery::DialDecision::ForceDial
        );
    }
}
