use super::lane::Lane;
use super::topology::Topology;

const LANES: [(&str, u64, u32); 3] = [("peer-1", 1, 1), ("peer-2", 2, 5), ("peer-3", 3, 17)];

const MESH_P1: &[&str] = &["peer-2", "peer-3"];
const MESH_P2: &[&str] = &["peer-1", "peer-3"];
const MESH_P3: &[&str] = &["peer-1", "peer-2"];
const CENTER: &[&str] = &["peer-2", "peer-3"];
const LEAF: &[&str] = &["peer-1"];

#[must_use]
pub fn lanes_for(topology: &Topology) -> [Lane; 3] {
    let visible = |name: &'static str| -> &'static [&'static str] {
        match topology {
            Topology::FullMesh => match name {
                "peer-1" => MESH_P1,
                "peer-2" => MESH_P2,
                _ => MESH_P3,
            },
            Topology::Star { center } => {
                if name == *center {
                    CENTER
                } else {
                    LEAF
                }
            }
        }
    };
    LANES.map(|(name, own, clicks)| Lane::new(name, own, clicks, visible(name)))
}

#[cfg(test)]
mod tests {
    use super::Topology;
    use super::lanes_for;

    #[test]
    fn test_usage() {
        let lanes = lanes_for(&Topology::FullMesh);
        assert_eq!(lanes.len(), 3);
        assert_eq!(lanes[0].visible.len(), 2);
        let star = lanes_for(&Topology::Star { center: "peer-1" });
        assert_eq!(star[1].visible, &["peer-1"]);
        assert_eq!(star[0].visible.len(), 2);
    }
}
