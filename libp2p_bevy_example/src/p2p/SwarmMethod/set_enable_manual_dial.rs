use crate::p2p::Command;
use crate::p2p::Swarm;

pub fn set_enable_manual_dial(swarm: &mut Swarm, enabled: bool) {
    swarm
        .command_sender
        .try_send(Command::SetEnableManualDial(enabled))
        .ok();
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;
    use crate::p2p::Swarm;

    #[test]
    fn test_usage() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        let (mut swarm, _rx) = Swarm::new(config)?;
        swarm.set_enable_manual_dial(false);
        swarm.set_enable_manual_dial(true);
        Ok(())
    }
}
