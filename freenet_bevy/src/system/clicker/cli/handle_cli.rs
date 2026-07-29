use bevy::prelude::*;

use crate::structs::clicker::cli::cli_command::CliCommand;
use crate::structs::clicker::clicker_state::ClickerState;
use crate::system::clicker;

pub fn handle_cli(mut reader: MessageReader<CliCommand>, mut state: ResMut<ClickerState>) {
    for cmd in reader.read() {
        match cmd {
            CliCommand::Increment => {
                clicker::increment::increment(&mut state, 1);
                println!("> incremented to {}", state.count);
            }
            CliCommand::Status => {
                println!("> current count: {}", state.count);
            }
            CliCommand::Help => {
                println!("{}", CliCommand::help_text());
            }
            CliCommand::Quit => {
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
    use crate::structs::clicker::cli::cli_command::CliCommand;
    use crate::structs::clicker::clicker_state::ClickerState;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<CliCommand>();

        let (tx, _rx) = mpsc::unbounded_channel();
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        app.insert_resource(ClickerState {
            event_rx: Mutex::new(mpsc::unbounded_channel().1),
            cmd_tx: tx,
            contract_key: key,
            count: 5,
        });

        app.add_systems(Update, handle_cli);

        app.world_mut()
            .resource_mut::<Messages<CliCommand>>()
            .write(CliCommand::Status);
        app.world_mut()
            .resource_mut::<Messages<CliCommand>>()
            .write(CliCommand::Increment);
        app.update();

        let state = app.world().resource::<ClickerState>();
        assert_eq!(state.count, 6);
    }
}
