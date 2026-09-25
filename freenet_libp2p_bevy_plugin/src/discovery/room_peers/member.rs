use super::discovery_status::DiscoveryStatus;
use crate::discovery;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub presence: discovery::id::Presence,
    pub status: DiscoveryStatus,
}

#[cfg(test)]
mod tests {
    use super::{DiscoveryStatus, Member};
    use crate::discovery;

    #[test]
    fn test_usage() {
        let member = Member {
            presence: discovery::id::Presence {
                addrs: Vec::new(),
                updated_at: discovery::id::EpochSecs(1),
            },
            status: DiscoveryStatus::Known,
        };
        assert_eq!(member.status, DiscoveryStatus::Known);
    }
}
