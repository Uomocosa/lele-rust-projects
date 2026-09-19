use super::mesh::Mesh;
use super::mesh_route;

pub fn step(mesh: &mut Mesh) {
    for app in &mut mesh.apps {
        app.update();
    }
    mesh_route::route(mesh);
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::p2p;

    use super::step;
    use crate::clicker;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        step(&mut mesh);
        for app in &mut mesh.apps {
            assert!(
                app.world()
                    .resource::<p2p::Commands<clicker::CursorMsg>>()
                    .is_empty()
            );
        }
    }
}
