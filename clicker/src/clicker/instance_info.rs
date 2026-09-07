use bevy::prelude::Resource;

use freenet_libp2p_bevy_plugin::net_id;

#[derive(Resource, Debug, Clone)]
pub struct InstanceInfo {
    pub namespace: String,
    pub instance_tag: u32,
    pub own_id: net_id::NetworkId,
}

#[cfg(test)]
mod tests {
    use super::InstanceInfo;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let info = InstanceInfo {
            namespace: "ns".to_string(),
            instance_tag: 1,
            own_id: net_id::NetworkId(1),
        };
        assert_eq!(info.instance_tag, 1);
    }
}
