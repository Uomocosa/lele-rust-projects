use bevy::app::App;
use bevy::prelude::*;

use crate::clicker;

pub fn build(app: &mut App) {
    app.add_message::<clicker::CliCommand>();
    app.add_systems(
        Update,
        (
            clicker::bevy_systems::read_stdin,
            clicker::bevy_systems::handle_cli,
            clicker::bevy_systems::write_stdout,
        ),
    );
}
