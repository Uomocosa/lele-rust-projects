use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Mesh, PrimitiveTopology};

#[must_use]
pub fn cursor_mesh(scale: f32) -> Mesh {
    let tip = [0.0, 0.0, 0.0];
    let base_left = [0.0, -34.0 * scale, 0.0];
    let base_right = [26.0 * scale, -10.0 * scale, 0.0];
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![tip, base_left, base_right]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
    mesh.insert_indices(bevy::mesh::Indices::U32(vec![0, 1, 2]));
    mesh
}

#[cfg(test)]
mod tests {
    use super::cursor_mesh;
    use bevy::mesh::Mesh;

    #[test]
    fn test_usage() {
        let mesh = cursor_mesh(1.0);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        let big = cursor_mesh(1.18);
        assert!(big.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
    }
}
