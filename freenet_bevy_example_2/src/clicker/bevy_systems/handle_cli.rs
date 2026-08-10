use bevy::prelude::*;

use crate::clicker;

pub fn handle_cli(
    mut reader: MessageReader<clicker::CliCommand>,
    mut state: ResMut<clicker::State>,
) {
    for cmd in reader.read() {
        match cmd {
            clicker::CliCommand::Increment => {
                clicker::increment(&mut state, 1);
                println!("> incremented to {}", state.count);
            }
            clicker::CliCommand::Status => {
                println!("> current count: {}", state.count);
            }
            clicker::CliCommand::Help => {
                println!("{}", clicker::CliCommand::help_text());
            }
            clicker::CliCommand::Quit => {
                println!("> quitting...");
                std::process::exit(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use bevy::prelude::*;
    use tokio::sync::mpsc;

    use super::handle_cli;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<clicker::CliCommand>();

        let (tx, _rx) = mpsc::unbounded_channel();
        app.insert_resource(clicker::State {
            event_rx: Mutex::new(mpsc::unbounded_channel().1),
            cmd_tx: tx,
            contract_key: None,
            count: 5,
            status: clicker::ConnectionStatus::Connecting,
            log: std::collections::VecDeque::new(),
        });

        app.add_systems(Update, handle_cli);

        app.world_mut()
            .resource_mut::<Messages<clicker::CliCommand>>()
            .write(clicker::CliCommand::Status);
        app.world_mut()
            .resource_mut::<Messages<clicker::CliCommand>>()
            .write(clicker::CliCommand::Increment);
        app.update();

        let state = app.world().resource::<clicker::State>();
        assert_eq!(state.count, 6);
    }
}
