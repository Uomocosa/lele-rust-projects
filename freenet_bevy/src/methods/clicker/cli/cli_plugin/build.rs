use bevy::app::App;
use bevy::prelude::*;

use crate::structs::clicker::cli::cli_command::CliCommand;

pub fn build(app: &mut App) {
    app.add_message::<CliCommand>();
    app.add_systems(
        Update,
        (
            crate::system::clicker::cli::read_stdin::read_stdin,
            crate::system::clicker::cli::handle_cli::handle_cli,
            crate::system::clicker::cli::write_stdout::write_stdout,
        ),
    );
}
