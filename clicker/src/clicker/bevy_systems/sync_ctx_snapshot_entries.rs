use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

pub fn snapshot_entries(
    ctx: &mut clicker::bevy_systems::SyncCtx<'_, '_>,
    targets: &Query<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
) -> Vec<(net_id::NetworkId, i32)> {
    if ctx.tombstones.is_empty() {
        ctx.seen.clear();
    }
    for (_, player, _) in targets {
        if let Some(number) = player {
            ctx.seen.insert(**number);
        }
    }
    let mut entries = Vec::new();
    for (_, player, counter) in targets {
        if let Some(number) = player {
            entries.push((net_id::NetworkId(**number), **counter));
        }
    }
    for (logical, count) in ctx.tombstones.iter() {
        if !ctx.seen.contains(logical) {
            continue;
        }
        let id = net_id::NetworkId(*logical);
        let mut merged = false;
        for (known, known_count) in &mut entries {
            if *known == id {
                if *count > *known_count {
                    *known_count = *count;
                }
                merged = true;
                break;
            }
        }
        if !merged {
            entries.push((id, *count));
        }
    }
    entries
}

// no test_usage necessary
