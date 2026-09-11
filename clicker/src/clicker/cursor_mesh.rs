use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Mesh, PrimitiveTopology};

const TILT_DEGREES: f32 = 0.0;
const TIP: [f32; 2] = [0.0, 0.0];
const BASE_LEFT: [f32; 2] = [-27.85, 19.5];
const BASE_RIGHT: [f32; 2] = [-23.1, -15.56];

#[must_use]
pub fn cursor_mesh(scale: f32) -> Mesh {
    let angle = TILT_DEGREES.to_radians();
    let cos = angle.cos();
    let sin = angle.sin();
    let tip = rotate(TIP, cos, sin, scale);
    let base_left = rotate(BASE_LEFT, cos, sin, scale);
    let base_right = rotate(BASE_RIGHT, cos, sin, scale);
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

// needed helper: rotates a 2d point around the tip and lifts it to 3d
fn rotate(point: [f32; 2], cos: f32, sin: f32, scale: f32) -> [f32; 3] {
    let [x, y] = point;
    let scaled_x = x * scale;
    let scaled_y = y * scale;
    [
        cos.mul_add(scaled_x, -sin * scaled_y),
        sin.mul_add(scaled_x, cos * scaled_y),
        0.0,
    ]
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
