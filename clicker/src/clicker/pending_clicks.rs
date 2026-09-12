use bevy::prelude::Resource;

use crate::clicker;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct PendingClicks {
    pub items: Vec<clicker::PendingClick>,
    pub parked: u64,
}

#[cfg(test)]
mod tests {
    use super::PendingClicks;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut pending = PendingClicks::default();
        assert_eq!(pending.items.len(), 0);
        pending.items.push(clicker::PendingClick {
            sender: net_id::NetworkId(9),
            owner: net_id::NetworkId(1),
            delta: 1,
            absolute: false,
        });
        pending.parked = pending.parked.saturating_add(1);
        assert_eq!(pending.items.len(), 1);
    }
}
