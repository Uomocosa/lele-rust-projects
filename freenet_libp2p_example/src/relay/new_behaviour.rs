use crate::relay;

#[must_use]
pub fn new_behaviour() -> relay::behaviour::Behaviour {
    libp2p::ping::Behaviour::new(libp2p::ping::Config::new())
}

#[cfg(test)]
mod tests {
    use super::new_behaviour;

    #[test]
    fn test_usage() {
        let _ = new_behaviour();
    }
}
