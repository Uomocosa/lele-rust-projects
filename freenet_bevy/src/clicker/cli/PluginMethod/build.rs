use bevy::app::App;
use bevy::prelude::*;

pub fn build(app: &mut App) {
    app.add_systems(
        Update,
        (
            crate::clicker::cli::system::read_stdin::read_stdin,
            crate::clicker::cli::system::write_stdout::write_stdout,
        ),
    );
}
