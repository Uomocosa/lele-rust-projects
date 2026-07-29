use bevy::app::App;
use bevy::prelude::*;

use crate::clicker::cli::CliCommand;

pub fn build(app: &mut App) {
    app.add_message::<CliCommand>();
    app.add_systems(
        Update,
        (
            crate::clicker::cli::system::read_stdin::read_stdin,
            crate::clicker::cli::system::handle_cli::handle_cli,
            crate::clicker::cli::system::write_stdout::write_stdout,
        ),
    );
}
