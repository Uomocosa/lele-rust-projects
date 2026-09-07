use bevy::prelude::*;

use crate::clicker;

pub fn log_connected(info: Res<clicker::InstanceInfo>, lobby: Res<clicker::ActiveLobby>) {
    let info = info.into_inner();
    let lobby = lobby.into_inner();
    let hash = clicker::contract_code_hash();
    tracing::info!(
        "namespace={} instance_tag={} own_id={} lobby={} contract={} connected, running indefinitely",
        info.namespace,
        info.instance_tag,
        *info.own_id,
        **lobby,
        hex_hash(&hash)
    );
}

fn hex_hash(hash: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for byte in hash {
        let _ = std::fmt::Write::write_fmt(&mut out, format_args!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::{hex_hash, log_connected};
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        assert_eq!(hex_hash(&[0u8; 32]).len(), 64);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(clicker::InstanceInfo {
            namespace: "ns".to_string(),
            instance_tag: 1,
            own_id: net_id::NetworkId(1),
        });
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.add_systems(Update, log_connected);
        app.update();
    }
}
