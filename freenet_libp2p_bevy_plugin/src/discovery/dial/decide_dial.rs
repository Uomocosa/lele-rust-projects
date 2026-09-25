use super::super::constants;
use super::decision::Decision;

#[must_use]
pub fn decide_dial(own_peer_id: &str, peer_id: &str, first_seen_secs_ago: Option<u64>) -> Decision {
    let retry = first_seen_secs_ago.is_some();
    if own_peer_id >= peer_id {
        match first_seen_secs_ago {
            None => return Decision::Wait,
            Some(age) if age < constants::REDIAL_SECS => return Decision::Wait,
            _ => {}
        }
    }
    if retry {
        Decision::ForceDial
    } else {
        Decision::Dial
    }
}

#[cfg(test)]
mod tests {
    use super::decide_dial;
    use crate::discovery;

    #[test]
    fn test_usage() {
        assert_eq!(decide_dial("a", "b", None), discovery::dial::Decision::Dial);
        assert_eq!(decide_dial("b", "a", None), discovery::dial::Decision::Wait);
        assert_eq!(
            decide_dial("b", "a", Some(discovery::REDIAL_SECS)),
            discovery::dial::Decision::ForceDial
        );
    }
}
