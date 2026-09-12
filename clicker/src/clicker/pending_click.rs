use freenet_libp2p_bevy_plugin::net_id;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingClick {
    pub sender: net_id::NetworkId,
    pub owner: net_id::NetworkId,
    pub delta: i32,
    pub absolute: bool,
}

#[cfg(test)]
mod tests {
    use super::PendingClick;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let item = PendingClick {
            sender: net_id::NetworkId(9),
            owner: net_id::NetworkId(1),
            delta: 1,
            absolute: false,
        };
        assert_eq!(item.delta, 1);
        assert!(!item.absolute);
    }
}
