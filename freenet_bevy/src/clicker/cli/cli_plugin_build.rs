use bevy::app::App;
use bevy::prelude::*;

use crate::clicker::cli::cli_command::CliCommand;

pub fn build(app: &mut App) {
    app.add_message::<CliCommand>();
    app.add_systems(
        Update,
        (
            crate::clicker::cli::read_stdin::read_stdin,
            crate::clicker::cli::handle_cli::handle_cli,
            crate::clicker::cli::write_stdout::write_stdout,
        ),
    );
}
