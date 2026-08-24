use freenet_libp2p_bevy_example_3_lib::{boxes, engine};

use crate::structs;

pub fn debug_snapshot(this: &structs::ProductionGameApp) -> String {
    let snapshot = this.app.world().resource::<boxes::LatestSnapshot>();
    match snapshot.as_ref() {
        Some(s) => {
            let bodies: Vec<String> = s
                .bodies
                .iter()
                .map(|(id, (x, y))| format!("{}:({:.4},{:.4})", hex_short(id), x, y))
                .collect();
            format!("tick={} [{}]", s.tick, bodies.join(", "))
        }
        None => "no snapshot".to_string(),
    }
}

// needed helper:
fn hex_short(id: &engine::PlayerId) -> String {
    id.iter().take(4).map(|b| format!("{b:02x}")).collect()
}

// no test_usage necessary - debug helper for determinism investigation
