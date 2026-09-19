pub enum Topology {
    Star { center: &'static str },
    FullMesh,
}
