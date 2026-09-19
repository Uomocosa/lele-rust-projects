#[derive(Clone, Copy)]
pub enum Nat {
    Open,
    NoHairpin { public_ip: &'static str },
}
