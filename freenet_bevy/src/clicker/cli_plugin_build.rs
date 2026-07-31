use bevy::app::App;
use bevy::prelude::*;

use crate::clicker::cli_command::CliCommand;

pub fn build(app: &mut App) {
    app.add_message::<CliCommand>();
    app.add_systems(
        Update,
        (
            crate::clicker::systems::read_stdin::read_stdin,
            crate::clicker::systems::handle_cli::handle_cli,
            crate::clicker::systems::write_stdout::write_stdout,
        ),
    );
}
